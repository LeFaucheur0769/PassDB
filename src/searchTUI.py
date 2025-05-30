import src.tui as tui
import src.searcher as searcher
import os

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
    tui.printLogo()
    print("01) Save output to a file")
    print("02) Print output to the terminal")
    # print("03) ")
    print("99) Return to main menu")
    
    choice = str(input("  PassDB> "))
    if choice == "1":
        SaveOutput(config)
    if choice == "2":
        searcher.searcher(config)
    if choice == "99":
        tui.tui(config)
        
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
    tui.printLogo()
    print("Enter the name of the file you would like to save to: ")
    outputName = str(input("  PassDB> "))
    if config.get("debug") == True: print("outputName: ", outputName)
    if (checkIfFileExists(config, outputName) == True):
        tui.printLogo()
        print("This file already exists")
        print("01) Overwrite")
        print("02) Change name")
        print("03) Cancel")
        print("99) Return to main menu")
        OverwriteChoice = str(input("  PassDB> "))
        if OverwriteChoice == "1":
            startSearcher(config, outputName)
        if OverwriteChoice == "2":
            tui.printLogo()
            while (checkIfFileExists(config, outputName) == True):
                print("Enter the name of the file you would like to save to: ")
                outputName = str(input("  PassDB> "))
                if checkIfFileExists(config, outputName) == True:
                    print("This file already exists")
                else:
                    startSearcher(config, outputName)
        if OverwriteChoice == "3":
            searchMenu(config)
        if OverwriteChoice == "99":
            tui.tui(config)
    else:
        startSearcher(config, outputName)

def startSearcher(config, outputName):
    outputFile = outputName
    open(config.get("export_results_location") + "/" + outputName, "w").close
    searcher.searcher(config, outputFile)
    
    
def checkIfFileExists(config, outputFile):
    if os.path.exists(config.get("export_results_location") + "/" + outputFile):
        return True
    else:
        return False
    