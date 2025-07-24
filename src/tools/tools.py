import subprocess

import src.tui as tui


import questionary
from concurrent.futures import ThreadPoolExecutor, as_completed
import os
import time
import re   
import tempfile
import shutil

URL_EMAIL_PASS_PATH = os.path.abspath("src/tools/urlEmailPass.sh")
DUPLICATE_FINDER_ALREADY_SORTED_PATH = "DuplicateFinderAlreadySorted.sh"
MOVE_TO_OTHER_PASS_DB_PATH = os.path.abspath("src/tools/appendFILE.sh")


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
        "Convert to usable files",
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
            ConvertToUsable()
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

def ConvertToUsable():
    print(f"You will be asked to choose a the format of the files you want to convert               ")
    print(f"To make the process faster, you have to only have one format of file in the folder      ")
    print(f"For storage reasons, only the email and the passwords will be kepts after this process  ")
    time.sleep(1)
    inputPath = input(str(f"Location of the files to sort: "))
    outputPath = input(str(f"Location of the sorted files: "))
    sortChoises = [
        "pass:email:anything",
        "anything:email:pass",
        "Return to tools menu",
        "Exit"
    ]
    while True:
        answer = questionary.select(
            choices=sortChoises,
            message="  PassDB> ",
        ).ask()
        if sortChoises.index(answer) == 0:  #pass:email:anything
            None
        elif sortChoises.index(answer) == 1:    #anything:email:pass
            UrlEmailPassDIR(inputPath, outputPath)
        elif sortChoises.index(answer) == 2:    #Return to tools menu
            tools()
        elif sortChoises.index(answer) == 3:    #Exit
            print("Goodbye")
            break

def checkTxtFiles(config):
    """
    Checks if all files in a directory are text files.

    If not all files are text files, the user is asked if they want to move the
    non-text files to another directory to sort them.

    :param config: The configuration dictionary.
    :return: None
    """
    inputPath = input(str(f"Location of the files to check: "))
    # Check if all files in the directory are text files
    answer = scan_directory_for_text_files(inputPath)
    if answer == True:
        print("All files are txt files")
    else:
        print("Not all files are txt files")
        # Ask the user if they want to move the non-text files to another directory
        move = input("Do you want to move the files to another directory to sort them ? (y/n)")
        if move == "y":
            # Move the files
            for file in answer:
                print(f"Moving {file} to {config.get('fiel_to_sort_not_txt_files')}")
                os.rename(file, os.path.join(config.get("fiel_to_sort_not_txt_files"), os.path.basename(file)))

def scan_directory_for_text_files(input_dir):
    """
    Scans a directory for text files and identifies non-text files.

    :param input_dir: The directory to scan for text files.
    :return: List of non-text file paths if any, otherwise True if all files are text files.
    """
    not_text_files = []  # List to store paths of non-text files

    # Walk through the directory
    for dirpath, dirnames, filenames in os.walk(input_dir):
        for filename in filenames:
            full_path = os.path.join(dirpath, filename)  # Full file path

            # Check if the file is a text file
            if allTxt(full_path):
                print(f"[TEXT] {full_path}")  # Print text file path
            else:
                print(f"[NOT TEXT] {full_path}")  # Print non-text file path
                not_text_files.append(full_path)  # Add to non-text files list

    # Return the list of non-text files or True if all are text files
    return not_text_files if not_text_files else True
                

def allTxt(full_path, num_bytes=10000):
    """
    Checks if a file is a text file by reading a chunk of it and trying to decode it as UTF-8.

    If the number of bad bytes (i.e. bytes that couldn't be decoded) is less than or equal to half of the number of bytes read, the function returns True, indicating that the file is a text file.

    :param full_path: The full path of the file to check.
    :param num_bytes: The number of bytes to read from the file. Defaults to 10000.
    :return: True if the file is a text file, False otherwise.
    """
    try:
        # Set the maximum number of bad bytes to half of the number of bytes read
        max_bad_bytes = num_bytes // 2
        with open(full_path, "rb") as f:
            # Read a chunk of the file
            chunk = f.read(num_bytes)

            # Decode ignoring errors
            decoded = chunk.decode('utf-8', errors='ignore')

            # Heuristic: if we lost too many bytes, it's probably binary
            bad_bytes = len(chunk) - len(decoded)

            # Return True if the file is a text file, False otherwise
            return bad_bytes <= max_bad_bytes
    except Exception as e:
        # Print any errors
        print(e)
        
def deleteAdvertisements(config, inputFile):
    """
    Deletes the header and ads from a file.

    This function assumes that the first line of the file is a valid line.
    It reads the file line by line and finds the first valid line.
    It then moves the valid content from the first valid line to the beginning of the file.

    :param config: The configuration object.
    :param inputFile: The input file to process.
    :return: None
    """
    print(f"Processing: {inputFile}")
    offset = 0

    # Step 1: Find byte offset of first valid line
    with open(inputFile, 'r', encoding='utf-8', errors='ignore') as f:
        while True:
            pos = f.tell()  # Byte offset before reading the line
            line = f.readline()
            if not line:
                print("No valid line found.")
                return
            if re.match(r'^[^:]+:[^:]+', line.strip()):
                offset = pos
                break

    if offset == 0:
        print("File already starts correctly.")
        return

    # Step 2: Move valid content to beginning of file
    chunk_size = 1024 * 1024  # 1MB
    with open(inputFile, 'r+b') as f:
        f.seek(offset)
        rest_pos = 0
        while True:
            data = f.read(chunk_size)
            if not data:
                break
            f.seek(rest_pos)
            f.write(data)
            rest_pos += len(data)
            f.seek(offset + rest_pos)

        f.truncate(rest_pos)

    print(f"Removed {offset} bytes of header/ads.")
        

def UrlEmailPass(inputFile, outputFile):
    """
    Runs the urlEmailPass.sh script with the given input file and output file

    The user is asked for an input file and an output file. The urlEmailPass.sh script is then run with the given input and output files.

    :return: None
    """
    if inputFile == None:
        # Ask the user for an input file
        inputFile = str(input("Input file path: "))
    if outputFile == None:
        # Ask the user for an output file
        outputFile = str(input("Output file path: "))

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
                    futures[
                        executor.submit(
                            runSript, URL_EMAIL_PASS_PATH, filepath, outputfile
                        )
                    ] = filepath
        # Iterate over the completed futures
        for future in as_completed(futures):
            # Get the file path associated with the future
            filepath = futures[future]
            try:
                # Get the result of the future
                future.result()
            except Exception as exc:
                # If an exception was raised, print it
                print("%s generated an exception: %s" % (filepath, exc))
            else:
                # If no exception was raised, print the result
                print(str(filepath) + " processed successfully")


def UrlEmailPassDIR(inputDir, outputDir):
    """
    Runs the urlEmailPass.sh script in parallel on all the files in the given input directory and saves the output in the given output directory

    The user is asked for an input directory and an output directory. The urlEmailPass.sh script is then run in parallel on all the files in the given input directory and the output is saved in the given output directory.

    :return: None
    """
    if inputDir == None:
        # Ask the user for an input directory
        inputDir = str(input("Input directory path: "))
    if outputDir == None:
        # Ask the user for an output directory
        outputDir = str(input("Output directory path: "))
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
                futures[
                    executor.submit(runSript, URL_EMAIL_PASS_PATH, filepath, outputfile)
                ] = filepath
        # Iterate over the completed futures
        for future in as_completed(futures):
            # Get the file path associated with the future
            filepath = futures[future]
            try:
                # Get the result of the future
                future.result()
            except Exception as exc:
                # If an exception was raised, print it
                print("%s generated an exception: %s" % (filepath, exc))
            else:
                # If no exception was raised, print the result
                print(str(filepath) + " processed successfully")


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

