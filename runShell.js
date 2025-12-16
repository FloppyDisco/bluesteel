let CWD = "/home/";

// File system structure:
// Functions = executables
// Objects = directories
// Arrays = files (content as strings)
const fileSystem = {
  "/": {
    "home/": {
      "README.md": [
        "There's 30 years of files in here...",
        "",
        "I hope you know what you're doing!",
      ],
      schedule: [
        "> Tea with Lapsang - 8:30am",
        "> Relax at Daiye Spa - 10:00am",
        "> Pick up Matilda from the office - 6:30pm",
        "> Prepare for Show - 9:27pm",
        "> Kick HanSolo's ass (Location: Members Only) - 9:30pm",
      ],
      ".games/": {
        snake: [
          "Welcome to Snake Game!",
          "Use arrow keys to move",
          "Press q to quit",
        ],
      },
    },
    "bin/": {
      ".README.md": [
        "This directory contains all executable commands that you can run in this OS",
      ],
      ls: function (args) {
        let targetPath = CWD;
        let showHidden = false;
        let pathArg = null;

        // Parse arguments
        for (const arg of args) {
          if (arg === "-a") {
            showHidden = true;
          } else if (!arg.startsWith("-")) {
            pathArg = arg;
          }
        }

        // If path argument provided, resolve it
        if (pathArg) {
          targetPath = resolvePath(pathArg, CWD);
        }

        const targetDir = getDirectoryContents(targetPath);
        if (targetDir === null) {
          printToTerminal(
            `ls: cannot access '${pathArg}': No such file or directory\n`,
          );
          return;
        }

        let items = Object.keys(targetDir).sort();

        // Filter hidden files/directories unless -a flag is used
        if (!showHidden) {
          items = items.filter((item) => !item.startsWith("."));
        }

        if (items.length > 0) {
          printToTerminal(items.join("  ") + "\n");
        }
      },
      pwd: function (args) {
        printToTerminal(CWD + "\n");
      },
      cd: function (args) {
        if (args.length === 0) {
          CWD = "/home/";
          return;
        }

        const target = args[0];
        const newPath = resolvePath(target, CWD);

        // Check if directory exists
        if (directoryExists(newPath)) {
          CWD = newPath;
        } else {
          printToTerminal(`bash: cd: ${target}: No such file or directory\n`);
        }
      },
      whoami: function (args) {
        printToTerminal("D-Rock69\n");
      },
      which: function (args) {
        if (args.length === 0) {
          printToTerminal("which: missing operand\n");
          return;
        }

        const command = args[0];
        const binDir = getDirectoryContents("/bin/");

        if (binDir && typeof binDir[command] === "function") {
          printToTerminal(`/bin/${command}\n`);
        } else {
          printToTerminal(`which: no ${command} in (/bin)\n`);
        }
      },
      cat: function (args) {
        if (args.length === 0) {
          printToTerminal("cat: missing operand\n");
          return;
        }

        const filePath = args[0];
        let targetPath;

        // Resolve the path
        if (filePath.startsWith("/")) {
          targetPath = filePath;
        } else {
          targetPath = CWD + (CWD.endsWith("/") ? "" : "/") + filePath;
        }

        const item = getFileAtPath(targetPath);

        if (item === null) {
          printToTerminal(`cat: ${filePath}: No such file or directory\n`);
        } else if (typeof item === "function") {
          printToTerminal(`cat: ${filePath}: Is a binary file\n`);
        } else if (typeof item === "object" && !Array.isArray(item)) {
          printToTerminal(`cat: ${filePath}: Is a directory\n`);
        } else if (Array.isArray(item)) {
          // Print file contents
          item.forEach((line) => printToTerminal(line + "\n"));
        } else {
          printToTerminal(`cat: ${filePath}: Unknown file type\n`);
        }
      },
      touch: function (args) {
        if (args.length === 0) {
          printToTerminal("touch: missing file operand\n");
          return;
        }

        const fileName = args[0];

        // Don't allow absolute paths or paths with /
        if (fileName.includes("/")) {
          printToTerminal(
            "touch: only simple filenames supported in current directory\n",
          );
          return;
        }

        // Get current directory contents
        const currentDir = getDirectoryContents(CWD);
        if (!currentDir) {
          printToTerminal("touch: current directory not found\n");
          return;
        }

        // Check if file already exists
        if (currentDir[fileName] !== undefined) {
          printToTerminal(`touch: cannot touch '${fileName}': File exists\n`);
          return;
        }

        // Create empty file (empty array)
        currentDir[fileName] = [];
        printToTerminal(`Created file: ${fileName}\n`);
      },
      mkdir: function (args) {
        if (args.length === 0) {
          printToTerminal("mkdir: missing operand\n");
          return;
        }

        const dirName = args[0];

        // Don't allow absolute paths or paths with /
        if (dirName.includes("/")) {
          printToTerminal(
            "mkdir: only simple directory names supported in current directory\n",
          );
          return;
        }

        // Get current directory contents
        const currentDir = getDirectoryContents(CWD);
        if (!currentDir) {
          printToTerminal("mkdir: current directory not found\n");
          return;
        }

        // Check if directory already exists
        if (currentDir[dirName + "/"] !== undefined) {
          printToTerminal(
            `mkdir: cannot create directory '${dirName}': File exists\n`,
          );
          return;
        }

        // Also check if a file with the same name exists (without /)
        if (currentDir[dirName] !== undefined) {
          printToTerminal(
            `mkdir: cannot create directory '${dirName}': File exists\n`,
          );
          return;
        }

        // Create empty directory (empty object)
        currentDir[dirName + "/"] = {};
        printToTerminal(`Created directory: ${dirName}/\n`);
      },
      echo: function (args) {
        if (args.length === 0) {
          printToTerminal("\n");
          return;
        }

        // Join all args to handle the full command line
        const fullCommand = args.join(" ");

        // Check for output redirection
        let redirectType = null;
        let fileName = null;
        let text = fullCommand;

        if (fullCommand.includes(">>")) {
          const parts = fullCommand.split(">>");
          if (parts.length === 2) {
            text = parts[0].trim();
            fileName = parts[1].trim();
            redirectType = "append";
          }
        } else if (fullCommand.includes(">")) {
          const parts = fullCommand.split(">");
          if (parts.length === 2) {
            text = parts[0].trim();
            fileName = parts[1].trim();
            redirectType = "overwrite";
          }
        }

        // Remove quotes if present
        if (
          (text.startsWith('"') && text.endsWith('"')) ||
          (text.startsWith("'") && text.endsWith("'"))
        ) {
          text = text.slice(1, -1);
        }

        if (redirectType && fileName) {
          // Output redirection with path support
          let targetPath;
          let targetDir;
          let targetFileName;

          // Resolve the file path
          if (fileName.startsWith("/")) {
            // Absolute path
            targetPath = fileName;
          } else {
            // Relative path
            targetPath = CWD + (CWD.endsWith("/") ? "" : "/") + fileName;
          }

          // Split path into directory and filename
          const pathParts = targetPath.split("/");
          targetFileName = pathParts.pop();
          const targetDirPath =
            pathParts.join("/") + (pathParts.length > 1 ? "/" : "");

          // Get target directory
          targetDir = getDirectoryContents(targetDirPath);
          if (!targetDir) {
            printToTerminal(`echo: ${fileName}: No such file or directory\n`);
            return;
          }

          if (redirectType === "overwrite") {
            // Check if target is not a directory or executable
            if (targetDir[targetFileName + "/"] !== undefined) {
              printToTerminal(`echo: ${fileName}: Is a directory\n`);
              return;
            }
            if (typeof targetDir[targetFileName] === "function") {
              printToTerminal(`echo: ${fileName}: Is a binary file\n`);
              return;
            }

            // Create or overwrite file
            targetDir[targetFileName] = [text];
          } else if (redirectType === "append") {
            // Check if target exists and is a file
            if (targetDir[targetFileName + "/"] !== undefined) {
              printToTerminal(`echo: ${fileName}: Is a directory\n`);
              return;
            }
            if (typeof targetDir[targetFileName] === "function") {
              printToTerminal(`echo: ${fileName}: Is a binary file\n`);
              return;
            }

            // Create file if it doesn't exist, or append to existing
            if (!Array.isArray(targetDir[targetFileName])) {
              targetDir[targetFileName] = [];
            }
            targetDir[targetFileName].push(text);
          }
        } else {
          // Just print to terminal
          printToTerminal(text + "\n");
        }
      },
      rm: function (args) {
        if (args.length === 0) {
          printToTerminal("rm: missing operand\n");
          return;
        }

        let recursive = false;
        let filesToRemove = [];

        // Parse arguments
        for (const arg of args) {
          if (arg === "-r" || arg === "-R") {
            recursive = true;
          } else if (!arg.startsWith("-")) {
            filesToRemove.push(arg);
          }
        }

        if (filesToRemove.length === 0) {
          printToTerminal("rm: missing operand\n");
          return;
        }

        // Process each file/directory to remove
        for (const target of filesToRemove) {
          let targetPath;
          let targetDir;
          let targetName;

          // Resolve the path
          if (target.startsWith("/")) {
            // Absolute path
            targetPath = target;
          } else {
            // Relative path
            targetPath = CWD + (CWD.endsWith("/") ? "" : "/") + target;
          }

          // Split path into directory and name
          const pathParts = targetPath.split("/");
          targetName = pathParts.pop();
          const targetDirPath =
            pathParts.join("/") + (pathParts.length > 1 ? "/" : "");

          // Get parent directory
          targetDir = getDirectoryContents(targetDirPath);
          if (!targetDir) {
            printToTerminal(
              `rm: cannot remove '${target}': No such file or directory\n`,
            );
            continue;
          }

          // Check if target exists
          const isDirectory = targetDir[targetName + "/"] !== undefined;
          const isFile = targetDir[targetName] !== undefined && !isDirectory;

          if (!isDirectory && !isFile) {
            printToTerminal(
              `rm: cannot remove '${target}': No such file or directory\n`,
            );
            continue;
          }

          if (isDirectory) {
            if (!recursive) {
              printToTerminal(
                `rm: cannot remove '${target}': Is a directory\n`,
              );
              continue;
            }

            // Remove directory (recursive)
            delete targetDir[targetName + "/"];
            printToTerminal(`Removed directory: ${target}/\n`);
          } else {
            // Remove file or executable
            delete targetDir[targetName];
            const itemType =
              typeof targetDir[targetName] === "function"
                ? "executable"
                : "file";
            printToTerminal(`Removed ${itemType}: ${target}\n`);
          }
        }
      },
      ftp: async function (args) {
        printToTerminal("Connecting to Bluesteel FTP Server...\n");

        const result = await fetch("https://download.bluesteel.software", {
          method: "head",
          cache: "no-store",
        });

        if (result.ok) {
          printToTerminal("Connected!\n");
          printToTerminal("Redirecting to downloads portal...\n");
          setTimeout(() => {
            window.open("https://download.bluesteel.software");
          }, 500);
        } else {
          printToTerminal(
            "Error: failed to connect to FTP server: server offline.\n",
          );
        }
      },
    },
  },
};

function runShell() {
  const screenContainer = document.getElementById("screen-container");

  // Clear screen and set up terminal
  screenContainer.innerHTML = "";
  screenContainer.style.color = "#00FF00";
  screenContainer.style.fontSize = "12px";
  screenContainer.style.padding = "10px";
  screenContainer.style.overflow = "auto";

  // Create input area
  const terminalOutput = document.createElement("div");
  terminalOutput.id = "terminal-output";
  screenContainer.appendChild(terminalOutput);

  const inputLine = document.createElement("div");
  inputLine.style.display = "flex";
  inputLine.innerHTML =
    '<span id="prompt"><span style="color: #00AAFF;">/home/</span> $ </span><input type="text" id="command-input" style="background: transparent; border: none; color: #00FF00; outline: none; flex: 1; font-family: inherit;">';
  screenContainer.appendChild(inputLine);

  const commandInput = document.getElementById("command-input");

  // Handle enter key
  commandInput.addEventListener("keydown", function (event) {
    if (event.key === "Enter") {
      const command = commandInput.value.trim();
      executeCommand(command);
      commandInput.value = "";
      scrollToBottom();
    }
  });

  commandInput.focus();
}

function executeCommand(command) {
  const output = document.getElementById("terminal-output");
  const prompt = document.getElementById("prompt");

  const commandLine = document.createElement("div");
  commandLine.innerHTML = `<span style="color: #00AAFF;">${CWD}</span> $ ${command}`;
  output.appendChild(commandLine);

  const parts = command.split(" ");
  const cmd = parts[0];
  const args = parts.slice(1);

  if (cmd === "") {
    updatePrompt();
    return;
  }

  // Try to find and execute the command
  const executable = findExecutable(cmd);
  if (executable) {
    executable(args);
  } else {
    printToTerminal(`bash: ${cmd}: command not found\n`);
  }

  updatePrompt();
}

function findExecutable(cmd) {
  // Check if it's a full path
  if (cmd.startsWith("/")) {
    const item = getFileAtPath(cmd);
    if (typeof item === "function") {
      return item;
    }
    return null;
  }

  // Check in /bin/ directory for short commands
  const binDir = getDirectoryContents("/bin/");
  if (binDir && typeof binDir[cmd] === "function") {
    return binDir[cmd];
  }

  return null;
}

function getFileAtPath(path) {
  const parts = path.split("/").filter((p) => p);
  let current = fileSystem["/"];

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (current[part + "/"]) {
      current = current[part + "/"];
    } else {
      return null;
    }
  }

  const fileName = parts[parts.length - 1];
  return current[fileName] || null;
}

function resolvePath(targetPath, currentPath) {
  if (targetPath.startsWith("/")) {
    // Absolute path
    return targetPath.endsWith("/") ? targetPath : targetPath + "/";
  } else if (targetPath === "..") {
    // Parent directory
    const parts = currentPath.split("/").filter((p) => p);
    parts.pop();
    return "/" + parts.join("/") + (parts.length > 0 ? "/" : "");
  } else {
    // Relative path
    return (
      currentPath + (currentPath.endsWith("/") ? "" : "/") + targetPath + "/"
    );
  }
}

function getDirectoryContents(path) {
  const parts = path.split("/").filter((p) => p);
  let current = fileSystem["/"];

  for (const part of parts) {
    if (current[part + "/"]) {
      current = current[part + "/"];
    } else {
      return null;
    }
  }

  return current;
}

function directoryExists(path) {
  return getDirectoryContents(path) !== null;
}

function printToTerminal(text) {
  const output = document.getElementById("terminal-output");
  const lines = text.split("\n");

  lines.forEach((line, index) => {
    if (index === lines.length - 1 && line === "") return;

    const lineElement = document.createElement("div");
    lineElement.textContent = line;
    output.appendChild(lineElement);
  });
}

function scrollToBottom() {
  const screenContainer = document.getElementById("screen-container");
  screenContainer.scrollTop = screenContainer.scrollHeight;
}

function updatePrompt() {
  const prompt = document.getElementById("prompt");
  prompt.innerHTML = `<span style="color: #00AAFF;">${CWD}</span> $ `;
}
