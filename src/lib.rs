use wasm_bindgen::prelude::*;

mod filesystem;
mod shell;

use filesystem::{Command, FileNode, FileSystem, FileType};
use shell::Shell;

use FileType::*;


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

    let cwd_path = local.cwd.borrow().get_path();

    let shell = Shell::new(
        local,
        document.clone(),
        terminal_output.clone(),
        terminal_input.clone(),
        prompt.clone(),
        prompt_directory.clone(),
        command_line.clone(),
    );

    shell.update_prompt(&cwd_path);

    return Ok(shell);
}
