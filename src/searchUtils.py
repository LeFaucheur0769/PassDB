import src.tui as tui
import src.searcher as searcher
import os

def startSearcher(config, outputName):
    """
    Starts the searcher function with the modified config and the file path as
    arguments. It creates the file if it does not exist and then calls the
    searcher function.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `import_location`.
        outputName (str): The filename to save the results to. If the file
                          already exists, it will be overwritten.
    """
    outputFile = outputName
    open(config.get("export_results_location") + "/" + outputName, "w").close
    searcher.searcher(config, outputFile)
    
    
def checkIfFileExists(config, outputFile):
    """
    Checks if a file exists in the export_results_location directory.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `import_location`.
        outputFile (str): The filename to check for existence.

    Returns:
        bool: True if the file exists, False otherwise.
    """
    if os.path.exists(config.get("export_results_location") + "/" + outputFile):
        return True
    else:
        return False


# def searchMenu(config):
#     """
#     Menu for searching the database.

#     This function prints the search menu to the user and asks for their input.
#     Depending on the user's choice, it either calls the SaveOutput function or
#     the searcher function.

#     Args:
#         config (dict): Configuration dictionary containing paths for
#                        `db_location` and `import_location`.
#     """
#     tui.printLogo()
    
#     print("01) Save output to a file")
#     print("02) Print output to the terminal")
#     # print("03) ")
#     print("99) Return to main menu")
    
#     print("")
#     choice = str(input("  PassDB> "))
#     print("")
#     if choice == "1":
#         SaveOutput(config)
#     if choice == "2":
#         searcher.searcher(config, None)
#     if choice == "99":
#         tui.tui(config)
        
# def SaveOutput(config):
#     """
#     Menu for saving output to a file.

#     This function asks the user for the name of the file they would like to
#     save to. If the file already exists, it gives the user the option to
#     overwrite, change the name, or cancel. Otherwise, it sets the
#     outputFile variable to the specified file path and calls the searcher
#     function with the modified config and the file path as arguments.

#     Args:
#         config (dict): Configuration dictionary containing paths for
#                        `db_location` and `import_location`.
#     """
#     tui.printLogo()
#     print("Enter the name of the file you would like to save to: ")
#     print("")
#     outputName = str(input("  PassDB> "))
#     print("")
#     if config.get("debug") == True: print("outputName: ", outputName)
#     if (checkIfFileExists(config, outputName) == True):
#         tui.printLogo()
#         print("This file already exists")
#         print("01) Overwrite")
#         print("02) Change name")
#         print("03) Cancel")
#         print("99) Return to main menu")
#         print("")
#         OverwriteChoice = str(input("  PassDB> "))
#         print("")
#         if OverwriteChoice == "1":
#             startSearcher(config, outputName)
#         if OverwriteChoice == "2":
#             tui.printLogo()
#             while (checkIfFileExists(config, outputName) == True):
#                 print("Enter the name of the file you would like to save to: ")
#                 print("")
#                 outputName = str(input("  PassDB> "))
#                 print("")
#                 if checkIfFileExists(config, outputName) == True:
#                     print("This file already exists")
#                 else:
#                     startSearcher(config, outputName)
#         if OverwriteChoice == "3":
#             searchMenu(config)
#         if OverwriteChoice == "99":
#             tui.tui(config)
#     else:
#         startSearcher(config, outputName)