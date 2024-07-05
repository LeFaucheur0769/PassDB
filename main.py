import sorter
import searcher
def main():
    choice = str(input("To add a combolist type 1, to search a combolist type 2: "))
    if (choice == "1"):
        sorter.sorter()
    elif (choice == "2"):
        searcher.searcher()
    print("Done")


if __name__ == '__main__':
    main()
    