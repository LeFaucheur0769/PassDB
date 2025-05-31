import os
import yaml
import time

import src.sorter as sorter
import src.searcher as searcher
import src.tui as tui
def main():
    """
    The main entry point for PassDB.

    Loads the configuration from the config file, initializes the database and
    import directories if they do not exist, and then enters the text-based user
    interface.

    Returns:
        None
    """
    
    tui.printLogo()
    with open("passdb.yml", "r") as f:
        print("Loading the config file...")
        time.sleep(0.5)
        print("\033[2J")
        config = yaml.safe_load(f)
        if (os.path.exists("output") == False):
            firstTime(config)
        else: 
            pass
        tui.tui(config)
        f.close()


    
def firstTime (config):
    """
    Initializes the necessary directories and files for first-time PassDB users.

    This function creates the database directory specified in the `config` and 
    initializes an empty `hash_db.txt` file within it. If the import directory 
    does not exist, it creates that as well. Displays progress messages and clears 
    the screen upon completion.

    Args:
        config (dict): Configuration dictionary containing paths for `db_location`
                       and `import_location`.

    """

    print("It's your first time using PassDB")
    print("Creating the directories...")
    os.mkdir(config.get("db_location"))
    open(config.get("db_location") + "/hash_db.txt" , "w").close
    if(os.path.exists(config.get("import_location")) == False):
        os.mkdir(config.get("import_location"))
    if(os.path.exists(config.get("export_results_location")) == False):
        os.mkdir(config.get("export_results_location"))
    print("Done\n\n")
    time.sleep(2)
    print("\033[2J")



 



if __name__ == '__main__':
    main()
    