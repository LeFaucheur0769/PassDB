import subprocess
# from src import sorter as sorter
# from src import searchUtils as searchUtils
# from src import searcher as searcher
import questionary
from concurrent.futures import ThreadPoolExecutor, as_completed
import os

URL_EMAIL_PASS_PATH = os.path.abspath("src/tools/urlEmailPass.sh")
DUPLICATE_FINDER_ALREADY_SORTED_PATH = "DuplicateFinderAlreadySorted.sh"
MOVE_TO_OTHER_PASS_DB_PATH =  os.path.abspath("src/tools/mvToUbuntu.sh")

def tools():
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
    choices = ["Url Email Pass", "Url Email Pass DIR","Url Email Pass DIR txt only", "Move to other PassDB" ,"Exit"]
    while True:
        # Ask the user to select an option
        answer = questionary.select(
            choices=choices,
            message="  PassDB> ",
        ).ask()
        # Run the selected tool
        if choices.index(answer) == 0:
            UrlEmailPass()
        elif choices.index(answer) == 1:
            UrlEmailPassDIR_txt_only()
        elif choices.index(answer) == 2:
            UrlEmailPassDIR()
        elif choices.index(answer) == 3:
            moveToOtherPassDB()
        elif choices.index(answer) == 4:
            print("Goodbye")
            break
        
def UrlEmailPass():
    """
    Runs the urlEmailPass.sh script with the given input file and output file
    
    The user is asked for an input file and an output file. The urlEmailPass.sh script is then run with the given input and output files.
    
    :return: None
    """
    # Ask the user for an input file
    inputFile = str(input("Input file path: "))
    # Ask the user for an output file
    outputFile = str(input("Output file path: "))
    # Run the urlEmailPass.sh script with the given input and output files
    runSript(URL_EMAIL_PASS_PATH, inputFile, outputFile)
    
def UrlEmailPassDIR_txt_only():
    """
    Runs the urlEmailPass.sh script in parallel on all the files in the given input directory and saves the output in the given output directory
    
    The user is asked for an input directory and an output directory. The urlEmailPass.sh script is then run in parallel on all the files in the given input directory and the output is saved in the given output directory.
    
    :return: None
    """
    # Ask the user for an input directory
    inputDir = str(input("Input directory path: "))
    # Ask the user for an output directory
    outputDir = str(input("Output directory path: "))
    # Run the urlEmailPass.sh script in parallel on all the files in the given input directory
    with ThreadPoolExecutor(max_workers=8) as executor:
        # dict to keep track of the futures and their corresponding file paths
        futures = {}
        # Iterate over all the files in the given input directory
        for root, dirs, files in os.walk(inputDir):
            # Iterate over all the files in the given input directory
            for file in files:
                if ".txt" not in file:
                    continue
                else: 
                    # Construct the file path
                    filepath = os.path.join(root, file)
                    # Construct the output file path
                    outputfile = os.path.join(outputDir, file)
                    # Submit the task to the executor
                    futures[executor.submit(runSript, URL_EMAIL_PASS_PATH, filepath, outputfile)] = filepath
        # Iterate over the completed futures
        for future in as_completed(futures):
            # Get the file path associated with the future
            filepath = futures[future]
            try:
                # Get the result of the future
                future.result()
            except Exception as exc:
                # If an exception was raised, print it
                print('%s generated an exception: %s' % (filepath, exc))
            else:
                # If no exception was raised, print the result
                print(str(filepath) + ' processed successfully')

def UrlEmailPassDIR():
    """
    Runs the urlEmailPass.sh script in parallel on all the files in the given input directory and saves the output in the given output directory
    
    The user is asked for an input directory and an output directory. The urlEmailPass.sh script is then run in parallel on all the files in the given input directory and the output is saved in the given output directory.
    
    :return: None
    """
    # Ask the user for an input directory
    inputDir = str(input("Input directory path: "))
    # Ask the user for an output directory
    outputDir = str(input("Output directory path: "))
    # Run the urlEmailPass.sh script in parallel on all the files in the given input directory
    with ThreadPoolExecutor(max_workers=8) as executor:
        # dict to keep track of the futures and their corresponding file paths
        futures = {}
        # Iterate over all the files in the given input directory
        for root, dirs, files in os.walk(inputDir):
            for file in files:
                # Construct the file path
                filepath = os.path.join(root, file)
                # Construct the output file path
                outputfile = os.path.join(outputDir, file)
                # Submit the task to the executor
                futures[executor.submit(runSript, URL_EMAIL_PASS_PATH, filepath, outputfile)] = filepath
        # Iterate over the completed futures
        for future in as_completed(futures):
            # Get the file path associated with the future
            filepath = futures[future]
            try:
                # Get the result of the future
                future.result()
            except Exception as exc:
                # If an exception was raised, print it
                print('%s generated an exception: %s' % (filepath, exc))
            else:
                # If no exception was raised, print the result
                print(str(filepath) + ' processed successfully')
                
def moveToOtherPassDB():
    """
    Runs the moveToOtherPassDB.sh script with the given input file and output file
    
    The user is asked for an input file and an output file. The moveToOtherPassDB.sh script is then run with the given input and output files.
    
    :return: None
    """
    # Ask the user for an input file
    inputFile = str(input("Input directory path: "))
    # Ask the user for an output file
    outputFile = str(input("Output directory path: "))
    # Run the moveToOtherPassDB.sh script with the given input and output files
    runSript(MOVE_TO_OTHER_PASS_DB_PATH, inputFile, outputFile)


def RemoveDuplicatesAlreadySorted():
    """
    Runs the removeDuplicatesAlreadySorted.sh script with the given input file
    
    The user is asked for an input file. The removeDuplicatesAlreadySorted.sh script is then run with the given input file.
    
    :return: None
    """
    # Ask the user for an input file
    inputFile = str(input("Input file path: "))
    # Run the removeDuplicatesAlreadySorted.sh script with the given input file
    runSript(DUPLICATE_FINDER_ALREADY_SORTED_PATH, inputFile)

def runSript(script_path, input, output=None):
    """
    Runs the given script with the given input and output
    
    :param script_path: The path to the script to run
    :type script_path: str
    :param input: The input to pass to the script
    :type input: str
    :param output: The output file to save the result to
    :type output: str
    :return: None
    """
    cmd = [script_path, input]
    if output is not None:
        cmd.append(output)
    subprocess.run(cmd)
    

tools()