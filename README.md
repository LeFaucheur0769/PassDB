# PassDB

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Description

**PassDB** is a high-performance, Rust-based tool for parsing, deduplicating, and querying large credential datasets (combolists). It features a modern Terminal User Interface (TUI) for easy interaction and utilizes a powerful [DuckDB](https://duckdb.org/) backend for fast and efficient data management.

This is a complete rewrite of the original Python version, designed for speed, memory efficiency, and a superior user experience.

## ✨ Key Features

*   **⚡ High Performance:** Written in Rust and using DuckDB for data storage and querying, PassDB is optimized to handle massive files with millions of entries.
*   **🖥️ Interactive TUI:** A user-friendly, keyboard-driven terminal interface built with `ratatui` makes managing and searching your databases simple and intuitive.
*   **🤖 Command-Line Interface (CLI):** Supports headless operation for integration into scripts and automation workflows.
*   **📂 Intelligent File Parsing:** Automatically detects and parses various input formats:
    *   Standard `email:password`, `user:pass`, and `url:login:pass` combos.
    *   `android://` formatted strings.
    *   JSON files (single objects or arrays).
    *   CSV files with header detection (handles common delimiters like `,`, `;`, and `\t`).
    *   French-style CSV files (`M., Mme.` format).
*   **💾 Deduplication:** Uses MD5 hashing to avoid re-processing and re-importing the same file, saving time and disk space.
*   **🔍 Fast Search:** Quickly search for a specific email or username across millions of records.
*   **🛠️ Configurable:** Easily customize import/export paths and database location via a simple `passdb.yml` configuration file.
*   **🧩 Extensible:** Modular architecture allows for adding new parsers and features.

## 🚀 Getting Started

### Prerequisites

*   **Rust and Cargo:** Ensure you have a working Rust toolchain installed. If not, follow the instructions at [rustup.rs](https://rustup.rs/).
*   **DuckDB:** The app uses the DuckDB library. The build process will handle the dependencies, but you may need the system libraries (like `libduckdb-dev`) installed on your system.

### Installation

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/LeFaucheur0769/PassDB.git
    cd PassDB
    ```

2.  **Build the Application:**
    ```bash
    cargo build --release
    ```
    The compiled binary will be located at `target/release/passdb`.

3.  **(Optional) Install the Binary:**
    You can copy the binary to a directory in your `PATH` for easy access.
    ```bash
    sudo cp target/release/passdb /usr/local/bin/
    ```

### Configuration

PassDB relies on a `passdb.yml` configuration file in the same directory as the binary or specified via the `--config` flag.

On the first run, PassDB will create a default configuration file if one is not present. You can customize the following options:

```yaml
# passdb.yml - Example Configuration
debug: false
db_location: "db/"                # Where to store the processed database and hash files
create_db_in_toolFolder: true     # Keeps everything inside the project folder
import_location: "import/"        # Folder to watch for new combolists to import
export_results_location: "export/" # Where to save search results
print_result_export_file: false   # Print results to console when exporting
check_if_valid_combolist: true
nbr_of_check_per_file: 10
file_to_sort_location: "to_sort/"
file_to_sort_not_txt_files: "to_sort/not_txt/"
file_urlloginpass_dir: "to_sort/url_login_pass/"
add_file: false
```

## 🎮 Usage

PassDB can be used in two main modes: **Interactive (TUI)** and **Command-Line (CLI)**.

### Interactive Terminal User Interface (TUI)

Launch the interactive menu system:

```bash
./passdb -i
```

or

```bash
./passdb --interactive
```

From the main menu, you can:

*   **Add a Combolist:** Select files from your import directory to parse, hash (for deduplication), and load into the database.
*   **Search a Combolist:** Search the database for a specific email or username and view the results.
*   **Access Tools:** Clean duplicates and perform other maintenance tasks (Coming Soon).

#### TUI Controls (General)

*   `↑` / `↓` : Navigate menus.
*   `Enter` : Select an option.
*   `q` / `Esc` : Go back a screen or quit the application.

#### Add a Combolist (AddDbScreen)

When adding a combolist, you'll see the progress of the hashing and importing process.

*   **`p`** : Pause/Resume processing.
*   **`s`** : Skip the current file.
*   **`c`** : Clear the log display.
*   **`q`** / `Esc` : Stop processing and return to the main menu.

#### Search a Combolist (SearchDbScreen)

1.  Choose to "Print the output to the terminal".
2.  Press `e` to enter edit mode.
3.  Type the email or username you are searching for.
4.  Press `Enter` to start the search.
5.  The results will be displayed. Use `↑` / `↓` to scroll. Press `q` to go back.

### Command-Line Interface (CLI)

For non-interactive or scripted use, you can run PassDB directly with arguments.

**Basic Search Example:**
```bash
./passdb -e "admin@example.com" --config passdb.yml
```

**Help:**
```bash
./passdb --help
```

```
Simple program to greet a person

Usage: passdb [OPTIONS] --email <EMAIL>

Options:
  -c, --config <CONFIG>      Location of the config file [default: passdb.yml]
  -i, --interactive          Launch PassDB in interactive mode
  -d, --debug                Enable debug mode
  -v, --verbose              Enable verbose mode
  -q, --quiet                Disable the output
  -e, --email <EMAIL>        Specify an email address to verify
      --import <IMPORT>      Specify an import directory [default: import/]
  -o, --output <OUTPUT>      Specify an output file
  -t, --test                 Run the module test
  -h, --help                 Print help
  -V, --version              Print version
```

## 🧱 Architecture

The application is built with a modular architecture, consisting of several key components:

*   **`main.rs`**: The entry point. Parses command-line arguments using `clap`, initializes the application, and runs the TUI or CLI mode.
*   **`tui/`**: The terminal user interface built with `ratatui`, `crossterm`, and a screen-based navigation system.
*   **`sorter/`**: The core parsing and import logic.
    *   `sorter.rs`: Contains the logic for parsing different file formats (JSON, CSV, `:`-separated) and inserting them into the DuckDB database.
    *   `hash_file.rs`: Calculates MD5 hashes of files for deduplication.
*   **`search/`**: The search engine. Queries the DuckDB database efficiently for email or username matches.
*   **`init/`**: Handles initial setup, validating the configuration file, and creating necessary directories.
*   **`logging/`**: A simple, structured logging system used throughout the application for progress tracking and debugging.

## 🤝 Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue. Pull requests are even better. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
