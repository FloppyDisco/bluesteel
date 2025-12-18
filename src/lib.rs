use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

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

#[wasm_bindgen]
pub fn run_shell() {
    fn ls() {
        todo!()
    }

    fn cd() {
        todo!()
    }

    fn snake() {
        todo!()
    }

    struct Path {
        parts: Vec<String>,
    }

    struct Text {
        name: String,
        contents: Vec<String>,
    }

    struct Executable {
        name: String,
        function: fn(),
    }

    enum File {
        Text(Text),
        Executable(Executable),
    }

    struct Directory {
        name: String,
        directories: Vec<Directory>,
        files: Vec<File>,
    }

    struct FileSystem {
        root: Directory,
        cwd: Path,
    }

    let local = FileSystem {
        root: Directory {
            name: String::from(""),
            directories: vec![
                Directory {
                    name: String::from("home"),
                    directories: vec![Directory {
                        name: String::from(".games"),
                        directories: Vec::new(),
                        files: vec![File::Executable(Executable {
                            name: String::from("snake"),
                            function: snake,
                        })],
                    }],
                    files: vec![File::Text(Text {
                        name: String::from("README.md"),
                        contents: vec![
                            String::from("There's 30 years of files in here!"),
                            String::from(""),
                            String::from("I hope you know what you're doing..."),
                        ],
                    })],
                },
                Directory {
                    name: String::from("bin"),
                    directories: Vec::new(),
                    files: vec![
                        File::Executable(Executable {
                            name: String::from("ls"),
                            function: ls,
                        }),
                        File::Executable(Executable {
                            name: String::from("cd"),
                            function: cd,
                        }),
                    ],
                },
            ],
            files: Vec::new(),
        },
        cwd: Path {
            parts: vec![String::from("home")],
        },
    };

    let Some(window) = web_sys::window() else {
        console_log!("window not found");
        return;
    };

    let Some(document) = window.document() else {
        console_log!("document not found");
        return;
    };

    let Some(screen_container) = document.get_element_by_id("screen-container") else {
        console_log!("screen-container not found");
        return;
    };

    let Some(terminal_output) = document.get_element_by_id("terminal-output") else {
        console_log!("terminal-output could not found");
        return;
    };

    let Some(terminal_input) = document.get_element_by_id("terminal-input") else {
        console_log!("terminal-input could not found");
        return;
    };
    terminal_input.set_id("terminal-input");

    let Some(prompt) = document.get_element_by_id("prompt") else {
        console_log!("prompt could not found");
        return;
    };
    
    // todo!
    // set the prompt to the cwd of the filesystem
    // must add some kind of printable trait or something
    prompt.set_text_content(Some("user@bluesteel:~"));

    let Some(command_line) = document.get_element_by_id("command-line") else {
        console_log!("command-line could not found");
        return;
    };

    // focus the command line
    let command_line_element = command_line.dyn_into::<HtmlInputElement>().unwrap();
    let _ = command_line_element.focus().unwrap();
}
