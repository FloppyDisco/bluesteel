use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

use crate::filesystem::{Command, FileNode, FileSystem, FileType};
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

impl Shell {
    pub fn new(
        filesystem: FileSystem,
        document: Document,
        output: Element,
        input: Element,
        prompt: Element,
        prompt_directory: Element,
        command_line: Element,
    ) -> Self {
        let shell = Self {
            filesystem,
            document,
            output,
            input,
            prompt,
            prompt_directory,
            command_line,
        };

        shell.update_prompt(&shell.filesystem.cwd().borrow().get_path());

        shell
    }
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

        let node_ref = match self.filesystem.get_command(&command) {
            Some(node_ref) => node_ref,
            None => {
                self.print_output(format!("sh: {}: command not found", command))?;
                return Ok(());
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
            Command::Pwd => self.pwd(args)?,
            Command::Whoami => self.whoami(args)?,
            Command::Which => self.which(args)?,
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

    fn pwd(&self, args: Vec<&str>) -> Result<(), JsValue> {
        if !args.is_empty() {
            self.print_output("pwd: too many arguments".to_string())?;
            return Ok(());
        }
        let cwd = self.filesystem.cwd();
        self.print_output(format!("{}", cwd.borrow().get_path()))?;
        Ok(())
    }

    fn whoami(&self, args: Vec<&str>) -> Result<(), JsValue> {
        if !args.is_empty() {
            self.print_output("usage: whoami".to_string())?;
            return Ok(());
        }
        self.print_output("zoolander99".to_string())?;
        Ok(())
    }

    fn which(&self, args: Vec<&str>) -> Result<(), JsValue> {
        for arg in args {
            if let Some(node_ref) = self.filesystem.get_command(arg) {
                self.print_output(node_ref.borrow().get_path())?;
            }
        }

        Ok(())
    }

    fn cd(&mut self, args: Vec<&str>) -> Result<(), JsValue> {
        let cwd = self.filesystem.cwd();

        if args.is_empty() {
            self.filesystem.set_cwd(self.filesystem.home());
        } else {
            if args.len() > 1 {
                self.print_output(format!("cd: '{}': invalid argument", args.join(" ")))?;
                return Ok(());
            };

            if let Some(path) = args.get(0) {
                if path == &"-" {
                    self.filesystem.set_cwd(self.filesystem.previous());
                } else {
                    if let Some(node_ref) = self.filesystem.get_node_ref(path) {
                        match &node_ref.borrow().file_type {
                            Directory(_) => {
                                self.filesystem.set_cwd(node_ref.clone());
                            }
                            _ => {
                                self.print_output(format!("cd: {}: not a directory", path))?;
                                return Ok(());
                            }
                        }
                    } else {
                        self.print_output(format!("cd: {}: directory not found", path))?;
                        return Ok(());
                    }
                }
            }
        };

        self.filesystem.set_previous(cwd);
        self.update_prompt(&format!("{}", self.filesystem.cwd().borrow().get_path()));
        Ok(())
    }

    fn ls(&self, args: Vec<&str>) -> Result<(), JsValue> {
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
            nodes_to_print.push(self.filesystem.cwd());
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
                            self.print_output(format!("  {}/:", &node.get_path()))?;

                            if display_hidden || !node.name.starts_with('.') {
                                match &node.file_type {
                                    File(_) | Executable(_) => {
                                        self.print_output(format!("  {}", &node.name))?;
                                    }
                                    Directory(children) => {
                                        for child_ref in children {
                                            if display_hidden
                                                || !child_ref.borrow().name.starts_with('.')
                                            {
                                                self.print_output(format!(
                                                    "{}",
                                                    child_ref.borrow().name
                                                ))?;
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
