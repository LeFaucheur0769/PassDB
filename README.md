# PassDB

##  Description

**PassDB** is a Python tool designed to efficiently sort combolists and provide fast, easy retrieval of `email:password` combinations. It's ideal for parsing and managing credential dumps with minimal effort.

---

##  Installation

### 1. Clone the repository

```bash
git clone https://github.com/LeFaucheur0769/PassDB.git
cd PassDB
```

### 2. Set up the Python environment

```bash
python -m venv .venv
# On Windows:
.venv\Scripts\activate
# On Unix/macOS:
source .venv/bin/activate
```

### 3. Install dependencies

```bash
pip install -r requirements.txt
```

---

##  Getting Started

On the first run, PassDB will automatically create the necessary folders and files.
You can customize these paths and behaviors in the [configuration file](#gear-config).

---

##  Usage

> *Usage instructions coming soon...*

---

## ⚙ Config

PassDB is highly configurable via the `passdb.yml` file located in the project directory.

| Option                     | Description                                                                                                                         |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `db_location`              | Path where processed combolists (databases) will be stored                                                                          |
| `create_db_in_toolFolder`  | If `true`, PassDB stores databases inside the tool’s directory. If `false`, you will be prompted for a custom location on first run |
| `import_location`          | Path from which raw combolists are imported                                                                                         |
| `export_results_location`  | Directory where search results or exports are saved                                                                                 |
| `print_result_export_file` | If `true`, results will be printed to the terminal even if they're being exported                                                   |
| `quiet`                    | If `true`, disables interactivity and only prints results (useful for scripting)                                                    |

---

