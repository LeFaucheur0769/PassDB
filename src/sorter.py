import hashlib
import os
import threading

# Global variables
# Get the directory of the current script
script_dir = os.path.dirname(os.path.abspath(__file__))


# List of all possible characters
alphaNum = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"

# def check_dir(config):
#     script_dir = os.path.dirname(os.path.realpath(__file__))

#     if not os.path.exists(script_dir + "/output"):
#         os.makedirs(script_dir + "/output")
#     if not os.path.exists(script_dir + "/output/sorted"):
#         os.makedirs(script_dir + "/output/sorted")
#     if not os.path.exists(script_dir + "/import"):
#         os.makedirs(script_dir + "/import")

#     return

def check_dir(config):
    """
    Checks if the necessary directories for the database exist and creates them if not.

    Args:
        config (dict): Configuration dictionary containing paths for `db_location`
                       and `import_location`.

    Returns:
        None
    """
    # script_dir = os.path.dirname(os.path.realpath(__file__))

    if not os.path.exists(config.get("db_location")):
        os.makedirs(config.get("db_location"))
    if not os.path.exists(config.get("db_location") + "/sorted"):
        os.makedirs(config.get("db_location") + "/sorted")
    if not os.path.exists(config.get("import_location")):
        os.makedirs(config.get("import_location"))
    if not os.path.exists(config.get("db_location") + "/hash_db.txt"):
        open(config.get("db_location") + "/hash_db.txt" , "w").close
        
    return


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


def alpha2file(file, importPath, config):
    """
    This function reads all text files in the "import" directory, sorts the passwords
    in each file alphabetically, and appends the sorted passwords to the "output.txt"
    file in the "output" directory.

    Args:
        file (str): The name of the file to sort
        importPath (str): The path to the import folder

    Returns:
        None
    """
    print("Sorting files...")


    # Create an empty list to store the passwords
    password = []

    # Iterate over all files in the "import" directory
    try:
        # Check if the file is a text file
        if file.endswith(".txt"):
            # Open the file and read its contents
            with open(os.path.join(importPath, file), encoding='utf-8') as f:
                # Add the passwords from the file to the list
                password.extend([line.strip() for line in f if line.strip()])
    except Exception as error:
        print("An Error occured")
        if config.get("debug") == True :
            print(error)

    # Sort the passwords alphabetically
    password.sort()

    # New sorting to file
    for line in password :
        for char1 in alphaNum :
            if str(line[0]) == char1 :
                for char2 in alphaNum :
                    if str(line[1]) == char2 :
                        for char3 in alphaNum :
                            if str(line[2]) == char3 :
                                open(os.path.join("output", "sorted") + "\\" + char1 + char2 + char3 + ".txt", "a", encoding="utf-8").write(line + "\n")
                                pass



# Enlever la dernière ligne si elle est vide avant d'écrire dans le fichier

def test():
    """
    Prints the names of all text files in the "import" directory.

    This function can be used to test the sorter.py script by printing the names of all
    text files in the "import" directory. This can be useful for debugging purposes.

    Args: None

    Returns: None
    """
    dir = os.path.dirname(os.path.realpath(__file__))
    for filename in os.listdir(dir + "/import/"):
        if filename.endswith(".txt"):
            print(filename)

def old_file_output(script_dir, password, importPath):
        # Open the "output.txt" file in the "output" directory and append the sorted passwords
    with open(script_dir + "/output/output.txt", "a", encoding='utf-8') as output:
        # Write each password to a new line in the file
        for line in password:
            output.write(line + "\n")











def sorter(config):
    """
    This function checks if the file was already added to the database
    by calling the hasher() function. If the file was already added, it
    prints a message saying that the file was already added to the database.
    If the file was not already added, it calls the alpha2file() function
    to sort the file and add it to the database.

    Args: None

    Returns: None
    """
    check_dir(config)
    importPath = config.get("import_location")
    # filename = "hash_db.txt"
    # print(importPath)
    # print(os.path.join(importPath, filename))
    for eachfiles in os.listdir(config.get("import_location")):
        # Check if the file was already added to the database
        print("Checking if",eachfiles, "'s hash was already added to the database...")
        if hasher(eachfiles, importPath, config) == True: # the file was already added to the database
            # Print a message saying that the file was already added to the database
            print("\033[91mThe file was already added to the database\033[0m")
        else: 
            # Sort the file and add it to the database
            print("\033[92mSorting ",eachfiles," and adding it to the database\033[0m")
            alpha2file(eachfiles, importPath, config)

    """
    This is a test function. It is commented out because it is not being used.
    """    
    # test()
