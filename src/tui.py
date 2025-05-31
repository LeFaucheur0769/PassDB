import src.searchUtils as searchUtils
import src.sorter as sorter
import src.searcher as searcher
import os
import questionary

def tui(config):
    if config.get("dynamic_menus") == True:
        welcomeScreen()

        choices = ["Add a combolist", "Search a combolist", "Exit"]
        if config.get("dynamic_menus") == True:
            answer = questionary.select(
                choices=choices,
                message="  PassDB> ",
            ).ask()
        if (choices.index(answer) == 0):
            sorter.sorter(config)
        elif (choices.index(answer) == 1):
            searchMenu(config)
        elif (choices.index(answer) == 2):
            print("Goodbye")
            exit()
    else:
        tuiOld(config)





def tuiOld(config):
    """
    The main text-based user interface for PassDB.

    This function prints the text-based menu for PassDB, which includes options
    for adding a combolist, searching a combolist, and exiting the program. It
    takes the user's input and calls the appropriate function based on their
    choice.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `import_location`.

    """
    welcomeScreen()
    print("01)  Add a combolist")
    print("02)  Search a combolist")
    print("99) Exit")
    print("")
    choice = str(input("  PassDB> "))
    print("")
    if (choice == "1"):
        sorter.sorter(config)
    elif (choice == "2"):
        searchMenu(config)
    elif (choice == "99" or choice == "exit" or choice == "Exit" or choice == "EXIT" or choice == "q"):
        print("Goodbye")
        exit()
    print("Done")
    
def searchMenu(config):
    """
    Menu for searching the database.

    This function prints the search menu to the user and asks for their input.
    Depending on the user's choice, it either calls the SaveOutput function or
    the searcher function.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `import_location`.
    """
    printLogo()
    choices = ["Save output to a file", "Print output to the terminal", "Return to main menu"]
    answer = questionary.select(
        choices=choices,
        message="  PassDB> ",
    ).ask()
    if (choices.index(answer) == 0):
        SaveOutput(config)
    elif (choices.index(answer) == 1):
        searcher.searcher(config, None)
    elif (choices.index(answer) == 2):
        tui(config)

def searchMenuOld(config):
    """
    Menu for searching the database.

    This function prints the search menu to the user and asks for their input.
    Depending on the user's choice, it either calls the SaveOutput function or
    the searcher function.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `import_location`.
    """
    tui.printLogo()
    print("01) Save output to a file")
    print("02) Print output to the terminal")
    # print("03) ")
    print("99) Return to main menu")
    
    print("")
    choice = str(input("  PassDB> "))
    print("")
    if choice == "1":
        SaveOutput(config)
    if choice == "2":
        searcher.searcher(config, None)
    if choice == "99":
        tuiOld(config)

def SaveOutput(config):
    """
    Menu for saving output to a file.

    This function asks the user for the name of the file they would like to
    save to. If the file already exists, it gives the user the option to
    overwrite, change the name, or cancel. Otherwise, it sets the
    outputFile variable to the specified file path and calls the searcher
    function with the modified config and the file path as arguments.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `import_location`.
    """
    printLogo()
    print("Enter the name of the file you would like to save to: ")
    print("")
    outputName = str(input("  PassDB> "))
    print("")
    if config.get("debug") == True: print("outputName: ", outputName)
    if (searchUtils.checkIfFileExists(config, outputName) == True):
        printLogo()
        print("This file already exists")
        print("01) Overwrite")
        print("02) Change name")
        print("03) Cancel")
        print("99) Return to main menu")
        print("")
        OverwriteChoice = str(input("  PassDB> "))
        print("")
        if OverwriteChoice == "1":
            searchUtils.startSearcher(config, outputName)
        if OverwriteChoice == "2":
            tui.printLogo()
            while (searchUtils.checkIfFileExists(config, outputName) == True):
                print("Enter the name of the file you would like to save to: ")
                print("")
                outputName = str(input("  PassDB> "))
                print("")
                if searchUtils.checkIfFileExists(config, outputName) == True:
                    print("This file already exists")
                else:
                    searchUtils.startSearcher(config, outputName)
        if OverwriteChoice == "3":
            searchMenu(config)
        if OverwriteChoice == "99":
            tui.tui(config)
    else:
        searchUtils.startSearcher(config, outputName)


def printLogo():
    """
    Prints the logo of the application in ASCII art format.
    
    This function outputs a stylized logo using ASCII characters to the console.
    The logo includes decorative text art with various symbols and spaces.
    """
    os.system('clear')
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
    
def welcomeScreen ():
    os.system('clear')
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
    print("        Welcome to PassDB - By LeFaucheur0769        ")
    print("\t\t\t\t\t")