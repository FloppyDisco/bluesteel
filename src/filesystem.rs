use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::JsValue;

#[derive(Clone)]
pub enum Command {
    Ls,
    Cd,
    Pwd,
    Whoami,
    Which,
    Cat,
    Touch,
    Mkdir,
    Rm,
    Echo,
}

#[derive(Clone)]
pub enum FileType {
    File(Vec<String>),
    Executable(Command),
    Directory(Vec<Rc<RefCell<FileNode>>>),
}

pub struct FileNode {
    pub name: String,
    pub parent: Option<Rc<RefCell<FileNode>>>,
    pub file_type: FileType,
}

impl FileNode {
    pub fn new(
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
                let mut node = child.borrow_mut();
                
                match &node.parent {
                    Some(_) => (),
                    None => {
                        node.parent = Some(node_ref.clone());
                    }
                }
            }
        };

        node_ref
    }

    pub fn get_path(&self) -> String {
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

        if segments.len() == 1 && segments[0] == "" {
            "/".to_string()
        } else {
            segments.reverse();
            segments.join("/")
        }
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

pub struct FileSystem {
    root: Rc<RefCell<FileNode>>,
    home: Rc<RefCell<FileNode>>,
    cwd: Rc<RefCell<FileNode>>,
    previous: Rc<RefCell<FileNode>>,
}

impl FileSystem {
    pub(crate) fn new(
        root: Rc<RefCell<FileNode>>,
        home: Rc<RefCell<FileNode>>,
        cwd: Rc<RefCell<FileNode>>,
        previous: Rc<RefCell<FileNode>>,
    ) -> Self {
        Self {
            root,
            home,
            cwd,
            previous,
        }
    }

    pub(crate) fn root(&self) -> Rc<RefCell<FileNode>> {
        self.root.clone()
    }

    pub(crate) fn home(&self) -> Rc<RefCell<FileNode>> {
        self.home.clone()
    }

    pub(crate) fn cwd(&self) -> Rc<RefCell<FileNode>> {
        self.cwd.clone()
    }

    pub(crate) fn previous(&self) -> Rc<RefCell<FileNode>> {
        self.previous.clone()
    }

    pub(crate) fn set_cwd(&mut self, cwd: Rc<RefCell<FileNode>>) {
        self.cwd = cwd;
    }

    pub(crate) fn set_previous(&mut self, previous: Rc<RefCell<FileNode>>) {
        self.previous = previous;
    }

    pub(crate) fn get_command(&self, command: &str) -> Option<Rc<RefCell<FileNode>>> {
        if let Some(node_ref) = self.get_node_ref(command) {
            return Some(node_ref);
        }

        if command.chars().nth(0) != Some('/') {
            return self.get_node_ref(&("/bin/".to_string() + command));
        }

        None
    }

    pub(crate) fn get_node_ref(&self, path: &str) -> Option<Rc<RefCell<FileNode>>> {
        if path == "" {
            return Some(self.cwd());
        };
        if path == "/" {
            return Some(self.root());
        };

        let mut current: Rc<RefCell<FileNode>> = if path.chars().nth(0) == Some('/') {
            self.root()
        } else {
            self.cwd()
        };

        for segment in path.split("/") {
            current = match segment {
                "" | "." => current,
                ".." => current
                    .borrow()
                    .parent
                    .clone()
                    .unwrap_or_else(|| current.clone()),
                _ => {
                    let FileType::Directory(child_nodes) = &current.borrow().file_type else {
                        return None;
                    };

                    let Some(child) = child_nodes
                        .iter()
                        .find(|child| child.borrow().name == segment)
                    else {
                        return None;
                    };

                    child.clone()
                }
            };
        }

        Some(current.clone())
    }

    pub(crate) fn add_node(
        &mut self,
        parent_ref: Rc<RefCell<FileNode>>,
        node_ref: Rc<RefCell<FileNode>>,
    ) -> Result<(), JsValue> {
        let mut parent = parent_ref.borrow_mut();
        match &mut parent.file_type {
            FileType::Directory(child_nodes) => {
                if child_nodes
                    .iter()
                    .any(|child| child.borrow().name == node_ref.borrow().name)
                {
                    Err(JsValue::from_str("Node already exists"))
                } else {
                    child_nodes.push(node_ref);
                    child_nodes.sort_by(|a, b| a.borrow().name.cmp(&b.borrow().name));
                    Ok(())
                }
            }
            FileType::File(_) | FileType::Executable(_) => {
                Err(JsValue::from_str("Cannot add node to a file or executable"))
            }
        }
    }

    pub(crate) fn remove_node(&mut self, node_ref: Rc<RefCell<FileNode>>) -> Result<(), JsValue> {
        let node = node_ref.borrow_mut();
        let Some(parent_ref) = node.parent.clone() else {
            return Err(JsValue::from_str("Node does not have a parent"));
        };

        match &mut parent_ref.borrow_mut().file_type {
            FileType::File(_) | FileType::Executable(_) => {
                return Err(JsValue::from_str(
                    "Cannot remove node from a file or executable",
                ));
            }
            FileType::Directory(children) => {
                if let Some(index) = children
                    .iter()
                    .position(|child_ref| Rc::ptr_eq(child_ref, &node_ref))
                {
                    children.remove(index);
                    return Ok(());
                } else {
                    return Err(JsValue::from_str("Node not found"));
                };
            }
        }
    }
}
