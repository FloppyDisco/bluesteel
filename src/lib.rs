use wasm_bindgen::prelude::*;

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

// A simple hello world function
#[wasm_bindgen]
pub fn run_shell() {
    console_log!("Boot the computer!");
}

// A function that manipulates the DOM
#[wasm_bindgen]
pub fn set_text_content(id: &str, text: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.get_element_by_id(id).unwrap();
    element.set_text_content(Some(text));
    console_log!("Set text content of element '{}' to: {}", id, text);
}

// A function that creates a new element and adds it to the DOM
#[wasm_bindgen]
pub fn add_paragraph(parent_id: &str, text: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    // Create a new paragraph element
    let paragraph = document.create_element("p").unwrap();
    paragraph.set_text_content(Some(text));
    
    // Convert to HtmlElement for styling
    let html_element = paragraph.dyn_into::<web_sys::HtmlElement>().unwrap();
    html_element.style().set_property("color", "#00AAFF").unwrap();
    html_element.style().set_property("margin", "5px 0").unwrap();
    
    // Find the parent element and append the paragraph
    let parent = document.get_element_by_id(parent_id).unwrap();
    parent.append_child(&html_element).unwrap();
    
    console_log!("Added paragraph to '{}': {}", parent_id, text);
}

// A function that returns a value to JavaScript
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    console_log!("Adding {} + {} = {}", a, b, a + b);
    a + b
}
