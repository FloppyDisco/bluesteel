use wasm_bindgen::prelude::*;

mod filesystem;
mod shell;

use filesystem::{Command, FileNode, FileSystem, FileType};
use shell::Shell;

use FileType::*;

#[wasm_bindgen]
pub fn boot_shell() -> Result<Shell, JsValue> {
    let bin = FileNode::new(
        "bin",
        None,
        FileType::Directory(vec![
            FileNode::new("pwd", None, FileType::Executable(Command::Pwd)),
            FileNode::new("ls", None, FileType::Executable(Command::Ls)),
            FileNode::new("cd", None, FileType::Executable(Command::Cd)),
            FileNode::new("whoami", None, FileType::Executable(Command::Whoami)),
            FileNode::new("which", None, FileType::Executable(Command::Which)),
            FileNode::new("cat", None, FileType::Executable(Command::Cat)),
            FileNode::new("touch", None, FileType::Executable(Command::Touch)),
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
                FileType::Directory(vec![
                    FileNode::new(".test", None, FileType::File(vec![
                        "here is a file".to_string(),
                        "it contains stuff".to_string(),
                        "".to_string(),
                        "it's contents are strings".to_string()
                    ])),
                    FileNode::new("whoami2", None, FileType::Executable(Command::Whoami)),
                ]),
            ),
        ]),
    );
    let local = FileSystem::new(
        FileNode::new("", None, FileType::Directory(vec![home.clone(), bin])),
        home.clone(),
        home.clone(),
        home.clone(),
    );

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

    let shell = Shell::new(
        local,
        document.clone(),
        terminal_output.clone(),
        terminal_input.clone(),
        prompt.clone(),
        prompt_directory.clone(),
        command_line.clone(),
    );

    return Ok(shell);
}
