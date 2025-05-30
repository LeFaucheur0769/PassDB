import hashlib
import os
from concurrent.futures import ThreadPoolExecutor, as_completed
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
        open(config.get("db_location") + "/hash_db.txt" , "w").close
        


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
    with open(file_path, encoding='utf-8') as file2open:
        contents = file2open.read()
        # Calculate the MD5 hash of the file contents
        file_hash = hashlib.md5(contents.encode()).hexdigest()
        if config.get("debug") == True : print(file_hash)

    # Path to the hash database file
    # hash_db_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), "output", "hash_db.txt")
    hash_db_path = os.path.join(config.get("db_location"), "hash_db.txt")

    try:
        # Open the "hash_db.txt" file and read its contents
        with open(hash_db_path, encoding='utf-8') as hash_db:
            # Check if the hash already exists in the database
            if file_hash in hash_db.read():
                # Print a message saying that the file was already added to the database
                print("File already in database")
                return True
            else:
                # Add the hash to the "hash_db.txt" file
                print("Adding file to database")
                with open(hash_db_path, "a", encoding='utf-8') as hash_db_append:
                    hash_db_append.write(file_hash + "\n")
                return False
    except FileNotFoundError:
        # If the "hash_db.txt" file does not exist, create it and add the hash
        print("Creating hash database")
        with open(hash_db_path, "w", encoding='utf-8') as hash_db_new:
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
            with open(os.path.join(importPath, "import") + "/" + file) as f:
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
                with open(os.join(importPath + "/output/sorted/") + char + ".txt", "a", encoding='utf-8') as output:
                    output.write(line + "\n")

def alpha2fileOptimized (file, importPath, config):
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
        with open(input_path, "r", encoding="utf-8") as f:
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
            with open(output_file, "a", encoding="utf-8") as out:
                out.write("\n".join(lines) + "\n")

    except Exception as e:
        # Print an error message if there is a problem
        print(f"\033[91m[ERROR] Failed to process {file}\033[0m")

        # Print debug information if the 'debug' key is set
        if config.get("debug"):
            print(f"  ↳ {e}")



def sorter(config):
    """
    Checks if each file has already been added to the database via hasher().
    If not, uses ThreadPoolExecutor to sort them concurrently with alpha2fileOptimized().
    """
    check_dir(config)
    importPath = config.get("import_location")
    files = os.listdir(importPath)

    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = {}

        for eachfile in files:
            print(f"Checking if {eachfile}'s hash was already added to the database...")

            if hasher(eachfile, importPath, config) is True:
                print("\033[91mThe file was already added to the database\033[0m")
            else:
                print(f"\033[92mSorting {eachfile} and adding it to the database\033[0m")
                # Submit the sorting task to the thread pool
                futures[executor.submit(alpha2fileOptimized, eachfile, importPath, config)] = eachfile

        # Optionally wait for all threads to complete and handle exceptions
        for future in as_completed(futures):
            file = futures[future]
            try:
                future.result()  # This re-raises exceptions from the thread, if any
                print(f"✅ Done sorting {file}")
            except Exception as e:
                print(f"\033[91m❌ Error sorting {file}: {e}\033[0m")
    """
    This is a test function. It is commented out because it is not being used.
    """    
    # test()



# if __name__ == "__main__":
#     import yaml
#     with open("passdb.yml", "r") as f:
#         sorter(config=yaml.safe_load(f))
#         f.close()