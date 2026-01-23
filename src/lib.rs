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
            FileNode::new("mkdir", None, FileType::Executable(Command::Mkdir)),
            FileNode::new("rm", None, FileType::Executable(Command::Rm)),
            FileNode::new("echo", None, FileType::Executable(Command::Echo)),
        ]),
    );
    let volume = FileNode::new(
        "zip_disc",
        None,
        FileType::Directory(vec![
            FileNode::new(
                "README.md",
                None,
                FileType::File(vec![
                    "I can't figure out how to get this to work".to_string(),
                    "Maury said to move it from the zip disc or something".to_string(),
                ]),
            ),
            FileNode::new("ssh", None, FileType::Executable(Command::Whoami)),
        ]),
    );
    let home = FileNode::new(
        "home",
        None,
        FileType::Directory(vec![
            FileNode::new(
                "README.md",
                None,
                File(vec![
                    "There's 30 years of files in here,".to_string(),
                    "I hope you know what you're doing!".to_string(),
                ]),
            ),
            FileNode::new(
                "schedule",
                None,
                File(vec![
                    "daiye spa until - 4pm".to_string(),
                    "pick up Matilda and little Derek - 530pm".to_string(),
                ]),
            ),
            FileNode::new(".games", None, FileType::Directory(vec![])),
        ]),
    );
    let local = FileSystem::new(
        FileNode::new(
            "",
            None,
            FileType::Directory(vec![
                home.clone(),
                bin,
                FileNode::new(".npm", Some(volume.clone()), FileType::Directory(vec![])),
                FileNode::new(".ssh", None, FileType::Directory(vec![
                    FileNode::new("config", None, File(vec![
                        "Host company-server".to_string(),
                        "  HostName 168.68.6.86".to_string(),
                        "  User bluesteel".to_string(),
                        "  IdentityFile /.ssh/id_rsa".to_string(),
                    ])),
                    FileNode::new("id_rsa", None, File(vec![
                        "-----BEGIN RSA PRIVATE KEY-----".to_string(),
                        // encode something in the private key stuff
                        "MIIEowIBAAKCAQEA...".to_string(),
                        "-----END RSA PRIVATE KEY-----".to_string(),
                    ])),
                    FileNode::new("id_rsa.pub", None, File(vec![
                        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACA... drock@bluesteel.software".to_string(),
                    ])),
                ])),
            ]),
        ),
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
