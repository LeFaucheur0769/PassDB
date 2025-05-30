import hashlib
import os

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
    script_dir = os.path.dirname(os.path.realpath(__file__))

    if not os.path.exists(config.get("db_location")):
        os.makedirs(config.get("db_location"))
    if not os.path.exists(config.get("db_location") + "/sorted"):
        os.makedirs(config.get("db_location") + "/sorted")
    if not os.path.exists(config.get("import_location")):
        os.makedirs(config.get("import_location"))
        
    return


def hasher(filename):
    """
    This function checks if a file has already been added to the database by
    calculating its MD5 hash and checking if the hash exists in a file called
    "hash_db.txt". If the hash exists, the function returns True, indicating
    that the file was already added to the database. If the hash does not
    exist, the function adds the hash to the "hash_db.txt" file, adds the
    file to the database, and removes it from the "import" directory.

    Args: None

    Returns: bool
        True if the file was already in the database, False if it was added
    """

    # Get the directory of the current script
    script_dir = os.path.dirname(os.path.realpath(__file__)) + "/import"
    
    # Open the file and read its contents
    with open(script_dir + "/" + filename, encoding='utf-8') as file2open:
        contents = file2open.read()
        # Calculate the MD5 hash of the file contents
        hash = hashlib.md5(contents.encode()).hexdigest()
        print(hash)

    try:
        # Open the "hash_db.txt" file and read its contents
        hash_db = open(os.path.dirname(os.path.realpath(__file__)) + "/output/hash_db.txt", encoding='utf-8')

        # Check if the hash already exists in the database
        if hash in hash_db.read():
            # Print a message saying that the file was already added to the database
            print("File already in database")
            hash_db.close()
            file2open.close()
            return True
        else:
            # Add the hash to the "hash_db.txt" file
            print("Adding file to database")
            hash_db.close()
            hash_db = open(os.path.dirname(os.path.realpath(__file__)) + "/output/hash_db.txt", "a", encoding='utf-8')
            hash_db.write(hash + "\n")
            hash_db.close()
            file2open.close()
            return False
    except FileNotFoundError:
        # If the "hash_db.txt" file does not exist, create it and add the hash
        print("Creating hash database")
        hash_db = open(os.path.dirname(os.path.realpath(__file__)) + "/output/hash_db.txt", "w", encoding='utf-8')
        hash_db.write(hash + "\n")
        hash_db.close()
        file2open.close()

    

def firstalpha2file():
    print("Sorting files...")
    """
    This function reads all text files in the "import" directory, sorts the passwords
    in each file alphabetically, and appends the sorted passwords to the "output.txt"
    file in the "output" directory.

    Args: None

    Returns: None
    """
    # Get the directory of the current script
    script_dir = os.path.dirname(os.path.abspath(__file__))

    # Create an empty list to store the passwords
    password = []

    # Iterate over all files in the "import" directory
    for file in os.listdir(script_dir + "/import/"):
        # Check if the file is a text file
        if file.endswith(".txt"):
            # Open the file and read its contents
            with open(script_dir + "/import/" + file) as f:
                # Add the passwords from the file to the list
                password.extend([line.strip() for line in f if line.strip()])

    # Sort the passwords alphabetically
    password.sort()

    # New sorting to file
    for line in password :
        for char in alphaNum :
            if str(line[0]) == char :
                open (script_dir + "/output/sorted/" + char + ".txt", "a", encoding='utf-8').write(line + "\n")
                pass

    # old_file_output(script_dir, password)


def alpha2file(file):
    print("Sorting files...")
    """
    This function reads all text files in the "import" directory, sorts the passwords
    in each file alphabetically, and appends the sorted passwords to the "output.txt"
    file in the "output" directory.

    Args: None

    Returns: None
    """

    # Create an empty list to store the passwords
    password = []

    # Iterate over all files in the "import" directory
    try:
        # Check if the file is a text file
        if file.endswith(".txt"):
            # Open the file and read its contents
            with open(script_dir + "/import/" + file, encoding='utf-8') as f:
                # Add the passwords from the file to the list
                password.extend([line.strip() for line in f if line.strip()])
    except:
        print("An Error occured")

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
                                open(script_dir + "/output/sorted/" + char1 + char2 + char3 + ".txt", "a", encoding="utf-8").write(line + "\n")
                                pass




# Enlever la dernière ligne si elle est vide avant d'écrire dans le fichier

def test():
    dir = os.path.dirname(os.path.realpath(__file__))
    for filename in os.listdir(dir + "/import/"):
        if filename.endswith(".txt"):
            print(filename)

def old_file_output(script_dir, password):
        # Open the "output.txt" file in the "output" directory and append the sorted passwords
    with open(script_dir + "/output/output.txt", "a", encoding='utf-8') as output:
        # Write each password to a new line in the file
        for line in password:
            output.write(line + "\n")











def sorter():
    """
    This function checks if the file was already added to the database
    by calling the hasher() function. If the file was already added, it
    prints a message saying that the file was already added to the database.
    If the file was not already added, it calls the alpha2file() function
    to sort the file and add it to the database.

    Args: None

    Returns: None
    """
    check_dir()
    for eachfiles in os.listdir(script_dir + "/import/"):
        # Check if the file was already added to the database
        print("Checking if",eachfiles, "'s hash was already added to the database...")
        if hasher(eachfiles) == True: # the file was already added to the database
            # Print a message saying that the file was already added to the database
            print("\033[91mThe file was already added to the database\033[0m")
        else: 
            # Sort the file and add it to the database
            print("\033[92mSorting ",eachfiles," and adding it to the database\033[0m")
            alpha2file(eachfiles)

    """
    This is a test function. It is commented out because it is not being used.
    """    
    # test()
