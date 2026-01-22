use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

mod filesystem;
use filesystem::{Command, FileNode, FileSystem, FileType};

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

use FileType::*;

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
    pub fn execute(&mut self, input: &str) -> Result<(), JsValue> {
        self.print_command(input)?;

        // because input is gauranteed to be at least an empty string
        // calling .split().remove(0) cannot fail because there will always be atleast one item
        let mut args: Vec<&str> = input.split(" ").collect();
        let command = args.remove(0);

        if command == "" {
            return Ok(());
        }

        let node_ref = match self.filesystem.get_node_ref(&command) {
            Some(node_ref) => node_ref,
            None => {
                // command not found in the current directory
                // if the command is not a absolute path ('/starts/with/slash')
                // check for the command in the /bin/
                let bin_node = if command.chars().nth(0) != Some('/') {
                    self.filesystem
                        .get_node_ref(&("/bin/".to_string() + command))
                } else {
                    None
                };

                if let Some(node_ref) = bin_node {
                    node_ref
                } else {
                    self.print_output(format!("sh: {}: command not found", command))?;
                    return Ok(());
                }
            }
        };

        let content = &node_ref.borrow().file_type;
        let executable = match content {
            Executable(command) => command,
            File(_) => {
                self.print_output(format!("sh: {}: file is not executable", command))?;
                return Ok(());
            }
            Directory(_) => {
                self.print_output(format!("sh: {}: directory is not executable", command))?;
                return Ok(());
            }
        };

        match executable {
            Command::Ls => self.ls(args)?,
            Command::Cd => self.cd(args)?,
        }
        Ok(())
    }

    fn update_prompt(&self, prompt: &str) {
        self.prompt_directory.set_text_content(Some(prompt));
    }

    fn print_output(&self, text: String) -> Result<(), JsValue> {
        let div = self.document.create_element("div")?;
        div.set_attribute("style", "min-height: 1em;")?;
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

    fn cd(&mut self, args: Vec<&str>) -> Result<(), JsValue> {
        let cwd = self.filesystem.cwd.clone();

        if args.is_empty() {
            self.filesystem.cwd = self.filesystem.home.clone();
        } else {
            if args.len() > 1 {
                self.print_output(format!("cd: '{}': invalid argument", args.join(" ")));
                return Ok(());
            };

            if let Some(path) = args.get(0) {
                if path == &"-" {
                    self.filesystem.cwd = self.filesystem.previous.clone();
                } else {
                    if let Some(node_ref) = self.filesystem.get_node_ref(path) {
                        match &node_ref.borrow().file_type {
                            Directory(_) => {
                                self.filesystem.cwd = node_ref.clone();
                            }
                            _ => {
                                self.print_output(format!("cd: {}: not a directory", path));
                                return Ok(());
                            }
                        }
                    } else {
                        self.print_output(format!("cd: {}: directory not found", path));
                        return Ok(());
                    }
                }
            }
        };

        self.filesystem.previous = cwd;
        self.update_prompt(&format!("{}", self.filesystem.cwd.borrow().get_path()));
        Ok(())
    }

    fn ls(&self, args: Vec<&str>) -> Result<(), JsValue> {
        console_log!("executing: ls");

        let mut display_hidden: bool = false;
        let mut display_subdirectories: bool = false;
        let mut received_path_arg: bool = false;
        let mut nodes_to_print: Vec<Rc<RefCell<FileNode>>> = Vec::new();

        for arg in args {
            match arg {
                "-a" | "--all" => display_hidden = true,
                "-R" | "--recurse" => display_subdirectories = true,
                "-aR" | "-Ra" => {
                    display_hidden = true;
                    display_subdirectories = true;
                }
                _ => {
                    received_path_arg = true;
                    if let Some(node_ref) = self.filesystem.get_node_ref(&arg) {
                        nodes_to_print.push(node_ref);
                    } else {
                        self.print_output(format!("ls: {}: no such file or directory", arg))?;
                    };
                }
            }
        }

        // allow passing 'ls' or 'ls -aR'
        // there are args but no path provided
        if nodes_to_print.is_empty() && !received_path_arg {
            nodes_to_print.push(self.filesystem.cwd.clone());
        }

        let display_headers = &nodes_to_print.len() > &1usize;

        for node_ref in nodes_to_print {
            if display_headers {
                self.print_output(format!("{}/:", node_ref.borrow().name))?;
            }
            match &node_ref.borrow().file_type {
                FileType::File(_) | FileType::Executable(_) => {
                    self.print_output(format!("  {}", node_ref.borrow().name))?;
                }
                FileType::Directory(children) => {
                    for child_ref in children {
                        if display_hidden || !child_ref.borrow().name.starts_with('.') {
                            self.print_output(format!("  {}", child_ref.borrow().name.clone()))?;
                        }
                    }

                    if display_subdirectories {
                        let mut stack: VecDeque<Rc<RefCell<FileNode>>> = children
                            .iter()
                            .filter(|child_ref| {
                                matches!(child_ref.borrow().file_type, Directory(_))
                            })
                            .cloned()
                            .collect();

                        while let Some(node_ref) = stack.pop_front() {
                            let node = node_ref.borrow();
                            self.print_output(format!("  {}/:", &node.get_path()));

                            if display_hidden || !node.name.starts_with('.') {
                                match &node.file_type {
                                    File(_) | Executable(_) => {
                                        self.print_output(format!("  {}", &node.name));
                                    }
                                    Directory(children) => {
                                        for child_ref in children {
                                            if display_hidden
                                                || !child_ref.borrow().name.starts_with('.')
                                            {
                                                self.print_output(format!(
                                                    "{}",
                                                    child_ref.borrow().name
                                                ));
                                            }

                                            if let Directory(_) = child_ref.borrow().file_type {
                                                stack.push_back(child_ref.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
        FileType::Directory(vec![
            FileNode::new("ls", None, FileType::Executable(Command::Ls)),
            FileNode::new("cd", None, FileType::Executable(Command::Cd)),
        ]),
    );
    let home = FileNode::new(
        "home",
        None,
        FileType::Directory(vec![
            FileNode::new(".hidden", None, File(Vec::new())),
            FileNode::new("not_hidden", None, File(Vec::new())),
            FileNode::new(
                "user",
                None,
                FileType::Directory(vec![FileNode::new(".test", None, FileType::File(vec![]))]),
            ),
        ]),
    );
    let local = FileSystem {
        cwd: home.clone(),
        home: home.clone(),
        previous: home.clone(),
        root: FileNode::new("", None, FileType::Directory(vec![home, bin])),
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
