use std::{collections::HashMap, fmt};
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

enum Content {
    File(Vec<String>),
    Executable(fn(Vec<&str>) -> Result<(), JsValue>),
    Directory(HashMap<String, Content>),
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Content::File(_) => "File",
            Content::Executable(_) => "Executable",
            Content::Directory(_) => "Directory",
        };
        write!(f, "{}", name)
    }
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
        Path::new(&format!("{}/{}", root.path, relative.path))
    }

    fn segments(&self) -> Vec<&str> {
        self.path.split("/").collect()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/", self.path)
    }
}

struct FileSystem {
    contents: Content,
    cwd: Path,
}

impl FileSystem {
    fn get_content(&self, path: &Path) -> Option<&Content> {
        let mut content = &self.contents;
        for segment in path.segments() {
            // console_log!("segment: {}", segment);
            // console_log!("content: {}", content);
            // this method needs to handle the "" string segment because i think it will come up alot in edge cases
            content = match content {
                Content::Directory(dir) => dir.get(segment)?,
                _ => return None,
            };
        }
        Some(content)
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
        // console_log!("input; {}", input);
        // console_log!("command: {}", command);

        let content = if command.starts_with('/') {
            let command_path = Path::new(command);

            let Some(content) = self.filesystem.get_content(&command_path) else {
                self.print_output(format!("sh: {}: command not found", command))?;
                return Ok(());
            };

            content
        } else {
            let relative_path = Path::new(command);
            let command_path = Path::combine(&self.filesystem.cwd, &relative_path);

            // console_log!("{}", command_path);

            let content = match self.filesystem.get_content(&command_path) {
                Some(content) => content,
                None => {
                    let bin_path = Path::combine(&Path::new("/bin/"), &relative_path);
                    // console_log!("{}", bin_path);
                    match self.filesystem.get_content(&bin_path) {
                        Some(content) => content,
                        None => {
                            self.print_output(format!("sh: {}: command not found", command))?;
                            return Ok(());
                        }
                    }
                }
            };

            content
        };

        // console_log!("{}", content);

        let executable = match content {
            Content::Executable(function) => function,
            Content::File(_) => {
                self.print_output(format!("sh: {}: file is not executable", command))?;
                return Ok(());
            }
            Content::Directory(_) => {
                self.print_output(format!("sh: {}: directory is not executable", command))?;
                return Ok(());
            }
        };

        executable(args)?;

        Ok(())
    }

    // fn set_cwd(&mut self, path: Path) {
    //     // check that content at the path is a directory
    //     self.filesystem.cwd = Path::new("/bin/")
    // }

    fn get_cwd(&self) -> Result<&Content, JsValue> {
        self.filesystem
            .get_content(&self.filesystem.cwd)
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
}

#[wasm_bindgen]
pub fn boot_shell() -> Result<Shell, JsValue> {
    fn snake(_args: Vec<&str>) -> Result<(), JsValue> {
        Ok(())
    }

    fn ls(_args: Vec<&str>) -> Result<(), JsValue> {
        Ok(())
        // accepts:
        // a path
        // the -a flag
        // the -r flag

        // convert all paths to an absolute path
        // traverse the file_system to find the requested path
        // if the file does not exist print an error

        // print the contents of the directory
        // -a to print hidden files as well
        // -r to print the contents of any nested directories as well
    }

    let bin = Content::Directory(HashMap::from([(
        String::from("ls"),
        Content::Executable(ls),
    )]));

    let local_home = Content::Directory(HashMap::from([
        (
            String::from("README.md"),
            Content::File(vec![
                String::from("There's 30 years of files in here!"),
                String::from(""),
                String::from("I hope you know what you're doing..."),
            ]),
        ),
        (
            String::from("schedule"),
            Content::File(vec![
                String::from("> Tea with Lapsang - 8:30am"),
                String::from("> Relax at Daiye Spa - 10:00am"),
                String::from("> Pick up Matilda - 6:30pm (Location: the Derek Zoolander Center)"),
                String::from("> Prepare for Show - 9:27pm"),
                String::from(
                    "> Meet up with Billy Zane and kick HanSolo's ass! - 9:30pm (Location: Members Only)",
                ),
            ]),
        ),
        (
            String::from(".games"),
            Content::Directory(HashMap::from([(
                String::from("snake"),
                Content::Executable(snake),
            )])),
        ),
    ]));

    let local = FileSystem {
        contents: Content::Directory(HashMap::from([
            (String::from("bin"), bin),
            (String::from("home"), local_home),
        ])),
        cwd: Path::new("/home/"),
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

    shell.update_prompt(&format!("{}", shell.filesystem.cwd));

    return Ok(shell);
}
