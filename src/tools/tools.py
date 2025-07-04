import subprocess
# from src import sorter as sorter
# from src import searchUtils as searchUtils
# from src import searcher as searcher
import questionary

URL_EMAIL_PASS_PATH = "urlEmailPass.sh"
DUPLICATE_FINDER_ALREADY_SORTED_PATH = "DuplicateFinderAlreadySorted.sh"

def tools():
    choices = ["Url Email Pass", "Exit"]
    if True:
        answer = questionary.select(
            choices=choices,
            message="  PassDB> ",
        ).ask()
    if (choices.index(answer) == 0):
        UrlEmailPass()
    elif (choices.index(answer) == 2):
        print("Goodbye")
        exit()
        
def UrlEmailPass():
    inputFile = str(input("Input file path: "))
    outputFile = str(input("Output file path: "))
    runSript(URL_EMAIL_PASS_PATH, inputFile, outputFile)

def RemoveDuplicatesAlreadySorted():
    inputFile = str(input("Input file path: "))
    runSript(DUPLICATE_FINDER_ALREADY_SORTED_PATH, inputFile)

def runSript(script_path, input, output=None):
    cmd = [script_path, input]
    if output is not None:
        cmd.append(output)
    subprocess.run(cmd)
    

tools()