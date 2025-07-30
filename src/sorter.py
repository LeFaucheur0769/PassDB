import hashlib
import os
from concurrent.futures import ThreadPoolExecutor, as_completed
import random # used to check random lines in the files 
import re # Used to check if lines contain a valid combolist
import logging
import time

# Global variables
# Get the directory of the current script
script_dir = os.path.dirname(os.path.abspath(__file__))


# List of all possible characters
alphaNum = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"


def check_dir(config):
    """
    Checks if the necessary directories for the database exist and creates them if not.

    Args:
        config (dict): Configuration dictionary containing paths for `db_location`
                       and `import_location`.

    Returns:
        None
    """

    if not os.path.exists(config.get("db_location")):
        os.makedirs(config.get("db_location"))
    if not os.path.exists(config.get("db_location") + "/sorted"):
        os.makedirs(config.get("db_location") + "/sorted")
    if not os.path.exists(config.get("import_location")):
        os.makedirs(config.get("import_location"))
    if not os.path.exists(config.get("db_location") + "/hash_db.txt"):
        open(config.get("db_location") + "/hash_db.txt", "w").close


def hasher(filename, importPath, config):
    """
    Checks if a file has already been added to the database by calculating its MD5 hash.

    This function calculates the MD5 hash of a file and checks if the hash exists in "hash_db.txt".
    If the hash exists, it indicates the file was already added to the database. If not, the hash
    is added to "hash_db.txt", and the file is added to the database.

    Args:
        filename (str): Name of the file to check.
        importPath (str): Path to the import directory.

    Returns:
        bool: True if the file was already in the database, False if it was added.
    """
    # Construct the full path to the file
    file_path = os.path.join(importPath, filename)

    # Open the file and read its contents
    with open(file_path, encoding="utf-8", errors="ignore") as file2open:
        contents = file2open.read()
        # Calculate the MD5 hash of the file contents
        file_hash = hashlib.md5(contents.encode()).hexdigest()
        if config.get("debug"):
            print(file_hash)

    # Path to the hash database file
    # hash_db_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), "output", "hash_db.txt")
    hash_db_path = os.path.join(config.get("db_location"), "hash_db.txt")

    try:
        # Open the "hash_db.txt" file and read its contents
        with open(hash_db_path, encoding="utf-8", errors="ignore") as hash_db:
            # Check if the hash already exists in the database
            if file_hash in hash_db.read():
                # Print a message saying that the file was already added to the database
                if config.get("debug"): print("File already in database")
                return True
            else:
                # Add the hash to the "hash_db.txt" file
                if config.get("debug"): print("Adding file to database")
                with open(
                    hash_db_path, "a", encoding="utf-8", errors="ignore"
                ) as hash_db_append:
                    hash_db_append.write(file_hash + "\n")
                return False
    except FileNotFoundError:
        # If the "hash_db.txt" file does not exist, create it and add the hash
        print("Creating hash database")
        with open(hash_db_path, "w", encoding="utf-8", errors="ignore") as hash_db_new:
            hash_db_new.write(file_hash + "\n")
        return False


def firstalpha2file(importPath):
    """
    Reads all text files in the "import" directory, sorts the passwords
    in each file alphabetically, and appends the sorted passwords to the
    "output.txt" file in the "output" directory.

    Args:
        importPath (str): The path to the import folder

    Returns:
        None
    """
    print("Sorting files...")

    # Get the directory of the current script
    # script_dir = os.path.dirname(os.path.abspath(__file__))

    # Create an empty list to store the passwords
    password = []

    # Iterate over all files in the "import" directory
    for file in os.listdir(os.path.join(importPath, "import") + "/"):
        # Check if the file is a text file
        if file.endswith(".txt"):
            # Open the file and read its contents
            with open(
                os.path.join(importPath, "import", encoding="utf-8", errors="ignore")
                + "/"
                + file
            ) as f:
                # Add the passwords from the file to the list
                password.extend([line.strip() for line in f if line.strip()])

    # Sort the passwords alphabetically
    password.sort()

    # Iterate over the sorted passwords
    for line in password:
        # Iterate over the first character of each password
        for char in alphaNum:
            # Check if the first character of the password matches the current character
            if str(line[0]) == char:
                # Append the password to the corresponding file
                with open(
                    os.join(importPath + "/output/sorted/") + char + ".txt",
                    "a",
                    encoding="utf-8",
                    errors="ignore",
                ) as output:
                    output.write(line + "\n")


def runScript(input_path, output_dir, config):
    import subprocess
    import os

    OS = os.name
    if OS == "nt":
        c_binary = config.get("win_binary")
    elif OS == "posix":
        c_binary = config.get("c_binary")
    else:
        exit()
    print(input_path, " ", output_dir, " ", c_binary)
    try:
        # Run the script with arguments
        result = subprocess.run(
            [c_binary, input_path, output_dir],
            capture_output=True,  # captures stdout and stderr
            text=True,  # returns output as string instead of bytes
            check=True,  # raises exception if script fails
        )
        # print("Script output:")
        print(result.stdout)  # print the script's stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running script: {e}")
        print("Script stderr:")
        print(e.stderr)


def alpha2fileOptimized(file, importPath, config):
    from collections import defaultdict

    # if not file.endswith(".txt"):
    #     return
    # Print the name of the file being processed
    print(f"Sorting file: {file}")

    # Construct the path to the input file
    input_path = os.path.join(importPath, file)

    # Construct the path to the output directory
    output_dir = os.path.join(config.get("db_location", ""), "sorted")

    # Create the output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    runScript(input_path, output_dir, config)


def alpha2fileOptimizedOld(file, importPath, config):
    """
    Processes a text file by reading its lines and categorizing them based on the
    first three alphanumeric characters. Each group of lines is then written to a
    separate output file named after the prefix in a specified directory.

    The function only processes files with a '.txt' extension. It ignores lines
    that are empty and prefixes that are not alphanumeric. In case of an error
    during processing, it logs the error to the console. Debug information is
    provided if the config dictionary contains a truthy 'debug' key.

    Args:
        file (str): The name of the text file to process.
        importPath (str): The directory path where the text file is located.
        config (dict): A configuration dictionary that includes paths and
                       debugging options.

    Returns:
        None
    """

    from collections import defaultdict

    if not file.endswith(".txt"):
        return

    # Print the name of the file being processed
    print(f"Sorting file: {file}")

    # Construct the path to the input file
    input_path = os.path.join(importPath, file)

    # Construct the path to the output directory
    output_dir = os.path.join(config.get("db_location", ""), "sorted")

    # Create the output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)

    # Initialize a dictionary to store the categorized lines
    buckets = defaultdict(list)

    try:
        # Open the input file for reading
        with open(input_path, "r", encoding="utf-8", errors="ignore") as f:
            # Iterate over each line in the file
            for line in f:
                # Strip the line of whitespace
                line = line.strip()

                # Ignore empty lines
                if not line:
                    continue

                # Extract the first three alphanumeric characters
                key = line[:3].lower() if len(line) >= 3 else line.lower()

                # Ignore lines with non-alphanumeric prefixes
                if key.isalnum():
                    # Add the line to the appropriate bucket
                    buckets[key].append(line)

        # Iterate over the buckets and write the lines to the appropriate output file
        for key, lines in buckets.items():
            output_file = os.path.join(output_dir, f"{key}.txt")
            with open(output_file, "a", encoding="utf-8", errors="ignore") as out:
                out.write("\n".join(lines) + "\n")

    except Exception as e:
        # Print an error message if there is a problem
        print(f"\033[91m[ERROR] Failed to process {file}\033[0m")

        # Print debug information if the 'debug' key is set
        if config.get("debug"):
            print(f"  ↳ {e}")
            

def check_valid_combolist_filetype(config, file):
    """
    Check if the given file has a valid file extension

    Args:
        config (dict): Configuration dictionary containing
            'nbr_of_check_per_file' key
        file (str): The path to the file

    Returns:
        bool: True if the file has a valid extension, False otherwise
    """
    valid_extensions = [".txt", ""]  # Allow empty extension
    if file.lower().endswith(tuple(valid_extensions)):
        return True
    else:
        print(f"\033[91m[ERROR] {file} is not a text file\033[0m")
        still_process = input(f"Do you still want to process it (y/n) ? ")
        if still_process.lower() == "y":
            return True
        else:
            return False

def check_if_valid_combolist_format(config, file):
    """
    Checks if a file is in a valid combo list format by sampling random lines.

    This function reads random lines from a file and checks if they comply with
    a specific format, which is determined by a regular expression pattern.
    The function returns True if at least one valid formatted line is found or
    if 10 lines are manually confirmed as valid by the user; otherwise, it returns False.

    Args:
        config (dict): Configuration dictionary containing 'nbr_of_check_per_file' key
                       and 'debug' flag for debugging output.
        file (str): The path to the file to be checked.

    Returns:
        bool: True if the file has a valid combo list format, False otherwise.
    """
    try:
        # Counter for manually confirmed valid lines
        valid = 0

        # Loop over a specified number of checks
        for _ in range(config.get("nbr_of_check_per_file")):
            # Generate a random line number to check
            random_line = random.randint(1, round((file.size() / 256) / 200))
            if config.get("debug"):
                print(f"Random line: {random_line} for file: {file}")

            # Open the file and read the random line
            with open(file, "r", encoding="utf-8", errors="ignore") as f:
                line = f.readlines()[random_line]

                # Check if the line matches the expected format
                if bool(re.match(r"[^:]+:[^:]+", line)):
                    # If the line matches the expected format, return True
                    print(f"Returning True for line: {line}")
                    return True
                else:
                    print(f"Returning False for line: {line}")
                    # Ask the user to manually confirm if the line is an error
                    error_check = input("Is this an error? (y/n): ")
                    if error_check.lower() == "y":
                        valid += 1

        # Return True if 10 lines were manually confirmed as valid
        return valid == 10
    except Exception as e:
        print(f"\033[91m[ERROR] Failed to process {file}\033[0m")
        if config.get("debug"):
            print(f"  ↳ {e}")
        return False
        
        
def check_if_email_first(config, file):
    """
    Checks if a given file contains a valid combo list format where the email is first.

    Args:
        config (dict): Configuration dictionary containing values for `nbr_of_check_per_file` and `debug`.
        file (str): Path to the file to check.

    Returns:
        bool: True if the file contains a valid combo list format, False otherwise.
    """
    try:
        # Counter for manually confirmed valid lines
        valid = 0

        # Loop over a specified number of checks
        for i in range(config.get("nbr_of_check_per_file")):
            # Generate a random line number to check
            random_line = random.randint(1, round((file.size() / 256) / 200))
            if config.get("debug"):
                print(f"Random line: {random_line} for file: {file}")

            # Open the file and read the random line
            with open(file, "r", encoding="utf-8", errors="ignore") as f:
                line = f.readlines()[random_line]

                # Check if the line matches the expected format
                if bool(re.match(r"[^:]+:[^:]+", line)):
                    # If the line matches the expected format, return True
                    print(f"Returning True for line: {line}")
                    return True
                else:
                    print(f"Returning False for line: {line}")
                    # Ask the user to manually confirm if the line is an error
                    error_check = input("Is this an error? (y/n): ")
                    if error_check.lower() == "y":
                        valid += 1
                    else:
                        None

        # Return True if 10 lines were manually confirmed as valid
        return valid == 10
    except Exception as e:
        print(f"\033[91m[ERROR] Failed to process {file}\033[0m")
        if config.get("debug"):
            print(f"  ↳ {e}")

def move_file_to_sort(config, file):
    """
    Move a given file to a specified directory

    This function moves a given file to a specified directory. It checks if the
    directory exists, and creates it if it does not. It then moves the file to the
    directory. If there is an error, it prints a message and the exception.

    Args:
        config (dict): A configuration dictionary containing the paths for the
            database and file to sort directories.
        file (str): The path to the file to be moved.

    Returns:
        None
    """
    try:
        # Check if the database directory exists, and create it if it does not
        # This is done to ensure that the file is moved to a valid location
        if os.path.exists(config.get("file_to_sort_location")) is False:
            os.makedirs(config.get("file_to_sort_location"))

        # Move the file to the database directory
        # This is done to sort the file alphabetically by its first three alphanumeric characters
        print(f"Moving {file} to {config.get('file_to_sort_location')}")
        os.rename(file, os.path.join(config.get("file_to_sort_location"), file))

    except Exception as e:
        # If there is an error, print a message and the exception
        # This is done to ensure that any errors are logged and the program
        # does not crash
        print(f"\033[91m[ERROR] Failed to move {file} to {config.get('file_to_sort_location')}\033[0m")
        if config.get("debug"):
            print(f"  ↳ {e}")

    

def sorter(config):
    """
    Sort the files in the import directory by their first three alphanumeric characters
    and write them to the database directory.

    Args:
        config (dict): A configuration dictionary containing the paths for the import
        and database directories.
    """
    # Ensure the necessary directories exist
    check_dir(config)
    importPath = config.get("import_location")

    # Use a thread pool to process files concurrently
    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = {}

        # Traverse through the import directory
        for root, dirs, files in os.walk(importPath):
            for eachfile in files:
                filepath = os.path.join(root, eachfile)

                # # Skip non-ASCII files
                # if not is_ascii_file(filepath):
                #     print(filepath + " is not ASCII")
                #     continue

                # Check if the file was already added to the database
                print(f"Checking if {eachfile}'s hash was already added to the database...")

                if hasher(eachfile, root, config) is True:
                    print("\033[91mThe file was already added to the database\033[0m")
                else:
                    print(f"\033[92mSorting {eachfile} and adding it to the database\033[0m")

                    # Submit the sorting task to the thread pool
                    futures[executor.submit(alpha2fileOptimized, eachfile, root, config)] = eachfile

        # Handle the results of the completed tasks
        for future in as_completed(futures):
            file = futures[future]
            try:
                future.result()
                print(f"✅ Done sorting {file}")
            except Exception as e:
                print(f"\033[91m❌ Error sorting {file}: {e}\033[0m")


# if __name__ == "__main__":
#     import yaml
#     with open("passdb.yml", "r") as f:
#         sorter(config=yaml.safe_load(f))
#         f.close()
