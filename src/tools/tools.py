import subprocess

import src.tui as tui


import questionary
from concurrent.futures import ThreadPoolExecutor, as_completed
import os
from src.tools.combolistFileManipulation import *




def tools(config):
    """
    Allows the user to select between tools to run

    The user is given a menu with the following options:
        - Url Email Pass: runs the urlEmailPass.sh script and asks the user for an input file and an output file
        - Url Email Pass DIR: runs the urlEmailPass.sh script and asks the user for an input directory and an output directory
        - Exit: exits the program

    The program will keep running until the user selects the Exit option

    :return: None
    """
    # List of options for the user
    choices = [
        "Check if files are usable files",
        "Url Email Pass",
        "Check if files are txt files",
        "Url Email Pass DIR txt only",
        "Move to other PassDB",
        "Return to main menu",
        "Exit",
    ]
    while True:
        # Ask the user to select an option
        answer = questionary.select(
            choices=choices,
            message="  PassDB> ",
        ).ask()
        # Run the selected tool
        if choices.index(answer) == 0:
            checkUsableFiles(config)
        elif choices.index(answer) == 1:
            UrlEmailPass()
        elif choices.index(answer) == 2:
            checkTxtFiles(config)
        elif choices.index(answer) == 3:
            UrlEmailPassDIR()
        elif choices.index(answer) == 4:
            moveToOtherPassDB()
        elif choices.index(answer) == 5:
            tui()
        elif choices.index(answer) == 6:
            print("Goodbye")
            break

def checkUsableFiles(config):
    try:
        print(f"Be careful, the full check will remove invalid lines. It will actively remove files or line")
        print("I highly recommend that you have a backup of your files before running this tool")
        choices = [
            "Do full check",
            "Check if files are usable files",
            "Check if files need to be shortened of advertisements",
            "Check if files are txt files",
            "Return to tools menu",
            "Exit"
        ]
        while True:
            # Ask the user to select an option
            answer = questionary.select(
                choices=choices,
                message="  PassDB> ",
            ).ask()
            # Run the selected tool
            if choices.index(answer) == 0:
                dirPath = input(str(f"Location of the files to check: "))
                for dirpath, dirnames, filenames in os.walk(dirPath):
                    for filename in filenames:
                        full_path = os.path.join(dirpath, filename)
                        fullCheck(config, full_path)
            elif choices.index(answer) == 5:
                break
    except:
        pass
    
    