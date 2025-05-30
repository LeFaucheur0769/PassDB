import os
import yaml
import time

import sorter
import searcher
def main():
    printLogo()
    
    with open("passdb.yml", "r") as f:
        print("Loading the config file...")
        time.sleep(0.5)
        print("\033[2J")
        config = yaml.safe_load(f)
        if (os.path.exists("output/sorted") == False):
            firstTime(config)
        else: 
            pass
        tui(config)
        f.close()
    
def firstTime (config):
    print("It's your first time using PassDB")
    print("Creating the directories...")
    os.mkdir(config.get("db_location"))
    open(config.get("db_location") + "/hash_db.txt" , "w").close
    if(os.path.exists(config.get("import_location")) == False):
        os.mkdir(config.get("import_location"))
    print("Done\n\n")
    time.sleep(2)
    print("\033[2J")
    
def tui(config):
    printLogo()
    print("1)  Add a combolist")
    print("2)  Search a combolist")
    print("99) Exit")
    print("\n")
    choice = str(input("  PassDB> "))
    print("")
    if (choice == "1"):
        sorter.sorter(config)
    elif (choice == "2"):
        searcher.searcher(config)
    print("Done")

def printLogo():
    """
    Prints the logo of the application in ASCII art format.
    
    This function outputs a stylized logo using ASCII characters to the console.
    The logo includes decorative text art with various symbols and spaces.
    """

    print("\t\t\t\t\t")
    print("                                                     ")
    print(" ██▓███   ▄▄▄        ██████   ██████ ▓█████▄  ▄▄▄▄   ")
    print("▓██░  ██▒▒████▄    ▒██    ▒ ▒██    ▒ ▒██▀ ██▌▓█████▄ ")
    print("▓██░ ██▓▒▒██  ▀█▄  ░ ▓██▄   ░ ▓██▄   ░██   █▌▒██▒ ▄██")
    print("▒██▄█▓▒ ▒░██▄▄▄▄██   ▒   ██▒  ▒   ██▒░▓█▄   ▌▒██░█▀  ")
    print("▒██▒ ░  ░ ▓█   ▓██▒▒██████▒▒▒██████▒▒░▒████▓ ░▓█  ▀█▓")
    print("▒▓▒░ ░  ░ ▒▒   ▓▒█░▒ ▒▓▒ ▒ ░▒ ▒▓▒ ▒ ░ ▒▒▓  ▒ ░▒▓███▀▒")
    print("░▒ ░       ▒   ▒▒ ░░ ░▒  ░ ░░ ░▒  ░ ░ ░ ▒  ▒ ▒░▒   ░ ")
    print("░░         ░   ▒   ░  ░  ░  ░  ░  ░   ░ ░  ░  ░    ░ ")
    print("               ░  ░      ░        ░     ░     ░      ")
    print("\t\t\t\t\t")

if __name__ == '__main__':
    main()
    