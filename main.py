import os
import yaml
import time
import argparse

import src.sorter as sorter
import src.searcher as searcher
import src.tui as tui

VERSION = 1.0
CONFIG = "passdb.yml"

def main(*args):
    """
    The main entry point for PassDB.

    Loads the configuration from the config file, initializes the database and
    import directories if they do not exist, and then enters the text-based user
    interface.

    Returns:
        None
    """
    if len(sys.argv) == 1:
        PassDB(CONFIG)
    else:
        parser()
        # print(args[0])

def PassDB(CONFIG):
    """
    The main entry point for PassDB.

    Loads the configuration from the config file, initializes the database and
    import directories if they do not exist, and then enters the text-based user
    interface.

    Returns:
        None
    """

    print("hi")
    tui.printLogo()
    try:
        with open(CONFIG, "r") as f:
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
    except yaml.YAMLError as exc:
        print(exc)

# def PassDB_quiet(CONFIG):
#     try:
#         with open(CONFIG, "r") as f:
#             config = yaml.safe_load(f)
#             f.close()
#     except yaml.YAMLError as exc:
#         print(exc)
#         exit()


    
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
    

def arguments(args):
    if args and (args[0] == "-h" or args[0] == "--help"):
        help()
        exit()
    if args and (args[0] == "-v" or args[0] == "--version"):
        print("PassDB version {f}", VERSION)
        exit()
    if args and (args[0] == "-c" or args[0] == "--config"):
        CONFIG = args[0]

def parser():
    """
    Parse command line arguments.

    This function parses the command line arguments and acts accordingly. 

    """
    parser = argparse.ArgumentParser()
    parser.add_argument("-c", "--config", type=str, help="Specify a configuration file")
    parser.add_argument("-d", "--debug", action="store_true", help="Enable debug mode")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose mode")
    parser.add_argument("-q", "--quiet", action="store_true", help="Disable output")
    parser.add_argument("-e", "--email", type=str, help="Specify an email address")
    parser.add_argument("-i", "--import", dest="import_dir", type=str, help="Specify an import directory")
    parser.add_argument("-o", "--output", type=str, help="Specify an output file")
    parser.add_argument("-V", "--version", action="store_true", help="Show version information and exit")
    args = parser.parse_args()
    
    if args.config:
        CONFIG = args.config
        PassDB(CONFIG)
    if args.version:
        print("PassDB version {}".format(VERSION))
        exit()
    if args.debug:
        print("Debug mode enabled")
    if args.verbose:
        print("Verbose mode enabled")
    if args.quiet:
        print("Quiet mode enabled")
    if args.email:
        print("Email address: {}".format(args.email))
    if args.import_dir:
        print("Import directory: {}".format(args.import_dir))
    if args.output:
        print("Output file: {}".format(args.output))

 



if __name__ == '__main__':
    import sys
    main(*sys.argv[1:])
    