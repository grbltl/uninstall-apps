# 🚀 Uninstall Apps

**Two Inputs:** directory location, such as / and a word to search for in the file/folder name, such as viewsonic
**Output:** A list of files/folder inside the directory specified in Input 1 whose name contains Input 2.

A fast macOS command-line tool that searches a directory for a word in file and folder names. Check the matches you want and delete the checked items together.

The search is case-insensitive and looks at the name only. It does not read file contents.

## ✨ Features

* **Parallel scan:** Uses every CPU core to walk large directory trees.
* **Live progress:** Shows directories and files scanned while the search runs.
* **Checkbox list:** Up and Down move the highlight. Space checks or unchecks the highlighted row.
* **Clipboard:** Checking a file or link copies its parent folder. Checking a folder copies that folder. A later check replaces the copied path. Unchecking clears the clipboard. The copy feature is helpful for manually deleting the stubborn ones.
* **Bulk delete:** Enter continues with every checked row. Delete removes those files, links, and folders after you type `yes`.
* **Skips locked folders:** Directories macOS will not let you read are passed over. The tool does not ask for `sudo`.

## 💻 Compatibility

The program compiles to native code for the Mac you build it on.

* **Intel (x86_64):** A binary built on Intel runs on other Intel Macs.
* **Apple Silicon (M series):** A binary built on Apple Silicon runs natively. An Intel binary runs through Rosetta 2. Build on the Apple Silicon Mac when you want native speed.

## 🛠️ Installation

### Option A: Download a release

1. Open [Releases](https://github.com/grbltl/uninstall-apps/releases).
2. Download `uninstall-apps`.
3. Make it executable and run it:

```bash
chmod +x uninstall-apps
./uninstall-apps
```

### Option B: Build from source

Building on your own Mac produces a binary for that Mac's processor.

1. Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

2. Clone the repository:

```bash
git clone https://github.com/grbltl/uninstall-apps.git
cd uninstall-apps
```

3. Build the release binary:

```bash
cargo build --release
```

The binary is `target/release/uninstall-apps`.

### Run it from any folder

```bash
sudo cp target/release/uninstall-apps /usr/local/bin/uninstall-apps
```

Then start it with `uninstall-apps`.

## 🎮 How to Use

1. Start the tool:

```bash
uninstall-apps
```

From a local build, run `./target/release/uninstall-apps`.

2. Enter the directory to search, then the word to find in each name.
3. In the result list:
   * **Up / Down** move between rows.
   * **Space** checks the highlighted row and copies its path, or unchecks it and clears the clipboard.
   * **Enter** continues with the checked rows.
   * **q** or **Esc** leaves without deleting. A path already copied stays on the clipboard.
4. Choose **Delete** or **Cancel**. Cancel is highlighted. Delete asks you to type `yes`.

A checked file is deleted as that file. A checked folder is deleted as that folder, including what is inside it. The parent folder copied to the clipboard is left in place when you delete a file.
