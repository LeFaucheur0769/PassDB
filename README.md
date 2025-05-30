# PassDB

## Installation

### Download the repository

```bash
git clone https://github.com/LeFaucheur0769/PassDB.git
cd PassDB
```

### Enable the python environnement 

```bash
python -m venv .venv
.venv\Scripts\activate    
```

### Install the dependencies

```bash
python -m pip install -r requirements.txt
```


## Run PassDB for the first time

If it is the first time you are running PassDB. It will automaticly create the necessary folders and files
You can specify the location of the folders and files in the [config file](#config)

## Usage

## Config

PassDB is customizable. You can edit it in the config file **passdb.yml**
- **db_location** : the location where the combolists are exported
- **create_db_in_toolFolder** : If the folders will be created in the tool's folder. If set to false, PassDB will ask a location on first boot
- **import_location** : the location from which the combolists are imported
- **export_results_location** : the location where the output are exported
- **print_result_export_file** : if set to true, PassDB will print the results even I you choose to export the output
- **quiet** : set to true to only print the results and disable the interactivity (useful if used in a script)
