use std::cell::{RefCell};
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

// Import the `console.log` function from the browser
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make console.log easier to use
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

enum Command {
    Pwd,
    Ls,
    Cd,
    Cat,
    Which,
    Whoami,
    Touch,
    Mkdir,
    Echo,
    Rm,
}

struct Path {
    path: String,
}

impl Path {
    fn new(path_str: &str) -> Self {
        let mut path = path_str.to_string();
        if path.chars().next() == Some('/') {
            path.remove(0);
        };
        if path.chars().next_back() == Some('/') {
            path.pop();
        };
        Path { path }
    }

    fn combine(root: &Path, relative: &Path) -> Self {
        // this naieve implementation does not allow for . and ..
        // this method will need to parse each part of the relative path
        // and manipulate the root for each segment
        Path::new(&format!("{}/{}", root.path, relative.path))
    }

    fn get_segments(&self) -> Vec<&str> {
        self.path.split("/").collect()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/", self.path)
    }
}

enum FileType {
    File(Vec<String>),
    Executable(Command),
    Directory(Vec<Rc<RefCell<FileNode>>>),
}

struct FileNode {
    name: String,
    parent: Option<Rc<RefCell<FileNode>>>,
    file_type: FileType,
}

impl FileNode {
    fn new(
        name: &str,
        parent: Option<Rc<RefCell<FileNode>>>,
        file_type: FileType,
    ) -> Rc<RefCell<FileNode>> {
        let node = FileNode {
            name: name.to_string(),
            parent,
            file_type,
        };

        let node_ref = Rc::new(RefCell::new(node));

        if let FileType::Directory(children) = &node_ref.borrow().file_type {
            for child in children {
                child.borrow_mut().parent = Some(node_ref.clone());
            }
        };

        node_ref
    }

    fn get_path(&self) -> Path {
        let mut segments: Vec<String> = Vec::new();
        
        let name = self.name.clone();
        segments.push(name);
        
        let mut ancestor = self.parent.clone();
        while let Some(node_ref) = ancestor {
            let node = node_ref.borrow();
            let segment = node.name.clone();
            segments.push(segment);
            ancestor = node.parent.clone()
        }

        segments.reverse();
        Path::new(&segments.join("/"))
    }
}

impl fmt::Display for FileNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.file_type {
            FileType::File(_) => "File",
            FileType::Executable(_) => "Executable",
            FileType::Directory(_) => "Directory",
        };
        write!(f, "{}", name)
    }
}

struct FileSystem {
    root: Rc<RefCell<FileNode>>,
    cwd: Rc<RefCell<FileNode>>,
}

impl FileSystem {
    // fn get_content(&self, path: &Path) -> Option<&FileNodeType> {
    //     let mut content = &self.contents;
    //     for segment in path.segments() {
    //         // console_log!("segment: {}", segment);
    //         // console_log!("content: {}", content);
    //         // this method needs to handle the "" string segment because i think it will come up alot in edge cases
    //         content = match content {
    //             FileNodeType::Directory(dir) => dir.get(segment)?,
    //             _ => return None,
    //         };
    //     }
    //     Some(content)
    // }
    fn get_node_ref(&self, path: &Path) -> Option<Rc<RefCell<FileNode>>> {
        let mut node_ref = self.root.clone();
        let segments = path.get_segments();

        for segment in segments {
            if segment.is_empty() {
                continue;
            }

            let clone = node_ref.clone();
            let FileType::Directory(children) = &clone.borrow().file_type else {
                return None;
            };

            if let Some(child_ref) = children
                .iter()
                .find(|node_ref| node_ref.borrow().name == segment)
            {
                node_ref = child_ref.clone();
            } else {
                return None;
            }
        }

        let node = node_ref.clone();
        Some(node)
    }
}

#[wasm_bindgen]
pub struct Shell {
    filesystem: FileSystem,
    document: Document,
    output: Element,
    input: Element,
    prompt: Element,
    prompt_directory: Element,
    command_line: Element,
}

#[wasm_bindgen]
impl Shell {
    pub fn execute(&self, input: &str) -> Result<(), JsValue> {
        self.print_command(input)?;

        // because input is gauranteed to be at least an empty string
        // calling .split().remove(0) cannot fail because there will always be atleast one item
        let mut args: Vec<&str> = input.split(" ").collect();
        let command = args.remove(0);

        if command == "" {
            return Ok(());
        }
        console_log!("input; {}", input);
        console_log!("command: {}", command);

        let node_ref = if command.starts_with('/') {
            let command_path = Path::new(command);

            let Some(node_ref) = self.filesystem.get_node_ref(&command_path) else {
                self.print_output(format!("sh: {}: command not found", command))?;
                return Ok(());
            };

            node_ref
        } else {
            let relative_path = Path::new(command);
            let command_path =
                Path::combine(&self.filesystem.cwd.borrow().get_path(), &relative_path);

            console_log!("{}", command_path);

            let node_ref = match self.filesystem.get_node_ref(&command_path) {
                Some(node_ref) => node_ref,
                None => {
                    let bin_path = Path::combine(&Path::new("/bin/"), &relative_path);
                    console_log!("{}", bin_path);
                    match self.filesystem.get_node_ref(&bin_path) {
                        Some(node_ref) => node_ref,
                        None => {
                            self.print_output(format!("sh: {}: command not found", command))?;
                            return Ok(());
                        }
                    }
                }
            };

            node_ref
        };

        console_log!("node: {}", node_ref.borrow());

        let content = &node_ref.borrow().file_type;
        let executable = match content {
            FileType::Executable(command) => command,
            FileType::File(_) => {
                self.print_output(format!("sh: {}: file is not executable", command))?;
                return Ok(());
            }
            FileType::Directory(_) => {
                self.print_output(format!("sh: {}: directory is not executable", command))?;
                return Ok(());
            }
        };

        match executable {
            Command::Ls => self.ls(args)?,
            _ => {}
        }
        Ok(())
    }

    // fn set_cwd(&mut self, path: Path) {
    //     // check that content at the path is a directory
    //     self.filesystem.cwd = Path::new("/bin/")
    // }

    fn generate_path(&self, path: &str) -> Path {
        Path::new(path)
    }

    fn get_cwd(&self) -> Result<Rc<RefCell<FileNode>>, JsValue> {
        self.filesystem
            .get_node_ref(&self.filesystem.cwd.borrow().get_path())
            .ok_or(JsValue::from("could not retrieve cwd"))
    }

    fn update_prompt(&self, prompt: &str) {
        self.prompt_directory.set_text_content(Some(prompt));
    }

    fn print_output(&self, text: String) -> Result<(), JsValue> {
        let div = self.document.create_element("div")?;
        div.set_text_content(Some(text.as_str()));
        self.output.append_child(&div)?;
        Ok(())
    }

    fn print_command(&self, text: &str) -> Result<(), JsValue> {
        let div = self.document.create_element("div")?;

        let prompt_directory = self
            .prompt_directory
            .clone_node_with_deep(true)?
            .dyn_into::<Element>()?;
        prompt_directory.remove_attribute("id")?;
        div.append_child(&prompt_directory)?;

        let line = self.document.create_element("span")?;
        line.set_text_content(Some(&(" $ ".to_owned() + text)));
        div.append_child(&line)?;
        self.output.append_child(&div)?;
        Ok(())
    }

    // // example cd ..
    // pub fn cd_up(&mut self) {
    //     if let Some(parent) = self.cwd.borrow().parent.clone() {
    //         self.cwd = parent;
    //     }
    // }

    fn ls(&self, args: Vec<&str>) -> Result<(), JsValue> {
        console_log!("executing: ls");
        
        let mut display_hidden: bool = false;
        let mut display_subdirectories: bool = false;
        let mut nodes_to_print: Vec<Rc<RefCell<FileNode>>> = Vec::new();
        
        if args.is_empty() {
            nodes_to_print.push(self.filesystem.cwd.clone());
        }
        for arg in args {
            match arg {
                "-a" => display_hidden = true,
                "-R" => display_subdirectories = true,
                "-aR" | "-Ra" => {
                    display_hidden = true;
                    display_subdirectories = true;
                }
                _ => {
                    let path = if arg.starts_with("/") {
                        Path::new(&arg)
                    } else {
                        Path::combine(&self.filesystem.cwd.borrow().get_path(), &Path::new(&arg))
                    };

                    if let Some(node_ref) = self.filesystem.get_node_ref(&path) {
                        nodes_to_print.push(node_ref);
                    } else {
                        self.print_output(format!("ls: {}: no such file or directory", arg))?;
                    };
                }
            }
        }

        for node_ref in nodes_to_print {
            match &node_ref.borrow().file_type {
                FileType::File(_) | FileType::Executable(_) => {
                    self.print_output(format!("{}",node_ref.borrow().name))?;
                }
                FileType::Directory(children) => {
                    for child_ref in children {
                        self.print_output(format!("{}", child_ref.borrow().name.clone()))?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub fn boot_shell() -> Result<Shell, JsValue> {
    // let bin = FileNodeType::Directory(HashMap::from([(
    //     String::from("ls"),
    //     FileNodeType::Executable(Command::Ls),
    // )]));

    // let local_home = FileNodeType::Directory(HashMap::from([
    //     (
    //         String::from("README.md"),
    //         FileNodeType::File(vec![
    //             String::from("There's 30 years of files in here!"),
    //             String::from(""),
    //             String::from("I hope you know what you're doing..."),
    //         ]),
    //     ),
    //     (
    //         String::from("schedule"),
    //         FileNodeType::File(vec![
    //             String::from("> Tea with Lapsang - 8:30am"),
    //             String::from("> Relax at Daiye Spa - 10:00am"),
    //             String::from("> Pick up Matilda - 6:30pm (Location: the Derek Zoolander Center)"),
    //             String::from("> Prepare for Show - 9:27pm"),
    //             String::from(
    //                 "> Meet up with Billy Zane and kick HanSolo's ass! - 9:30pm (Location: Members Only)",
    //             ),
    //         ]),
    //     ),
    //     // (
    //     //     String::from(".games"),
    //     //     FileNodeType::Directory(HashMap::from([(
    //     //         String::from("snake"),
    //     //         FileNodeType::Executable(Command::Snake),
    //     //     )])),
    //     // ),
    //     (
    //         String::from("downloads"),
    //         FileNodeType::Directory(HashMap::from([])),
    //     ),
    // ]));

    // let local = FileSystem {
    //     contents: FileNodeType::Directory(HashMap::from([
    //         (String::from("bin"), bin),
    //         (String::from("home"), local_home),
    //     ])),
    //     cwd: Path::new("/home/"),
    // };

    let bin = FileNode::new(
        "bin",
        None,
        FileType::Directory(vec![FileNode::new(
            "ls",
            None,
            FileType::Executable(Command::Ls)
        )])
    );
    let home = FileNode::new(
        "home",
        None,
        FileType::Directory(vec![FileNode::new(
            "user",
            None,
            FileType::Directory(vec![]),
        )]),
    );
    let local = FileSystem {
        cwd: home.clone(),
        root: FileNode::new("", None, FileType::Directory(vec![home, bin    ])),
    };

    let window = web_sys::window().ok_or("window not found")?;
    let document = window.document().ok_or("document not found")?;
    let _screen_container = document
        .get_element_by_id("screen-container")
        .ok_or("screen-container not found")?;
    let terminal_output = document
        .get_element_by_id("terminal-output")
        .ok_or("terminal-output could not found")?;
    let terminal_input = document
        .get_element_by_id("terminal-input")
        .ok_or("terminal-input could not found")?;
    let prompt = document
        .get_element_by_id("prompt")
        .ok_or("prompt could not found")?;
    let prompt_directory = document
        .get_element_by_id("prompt-directory")
        .ok_or("prompt-directory could not found")?;
    let command_line = document
        .get_element_by_id("command-line")
        .ok_or("command-line could not found")?;

    let shell = Shell {
        filesystem: local,
        document: document.clone(),
        output: terminal_output.clone(),
        input: terminal_input.clone(),
        prompt: prompt.clone(),
        prompt_directory: prompt_directory.clone(),
        command_line: command_line.clone(),
    };

    shell.update_prompt(&format!("{}", shell.filesystem.cwd.borrow().get_path()));

    return Ok(shell);
}
