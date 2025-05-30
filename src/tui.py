import src.searchTUI as searchTUI
import src.sorter as sorter

def tui(config):
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
    printLogo()
    print("01)  Add a combolist")
    print("02)  Search a combolist")
    print("99) Exit")
    print("")
    choice = str(input("  PassDB> "))
    print("")
    if (choice == "1"):
        sorter.sorter(config)
    elif (choice == "2"):
        searchTUI.searchMenu(config)
    elif (choice == "99" or choice == "exit" or choice == "Exit" or choice == "EXIT" or choice == "q"):
        print("Goodbye")
        exit()
    print("Done")


def printLogo():
    """
    Prints the logo of the application in ASCII art format.
    
    This function outputs a stylized logo using ASCII characters to the console.
    The logo includes decorative text art with various symbols and spaces.
    """
    print("\033[2J")
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