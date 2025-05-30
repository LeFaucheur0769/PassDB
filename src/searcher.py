import os
import src.tui as tui

def searcher(config, outputFile):
    """
    Searches the sorted database for a given email address.

    This function takes an email address as input and searches the sorted
    database for any lines containing that email address. If the email address is
    found, it prints the line from the database file. If the email address is not
    found, it prints a message to the console indicating that the email address
    was not found.

    Args:
        config (dict): Configuration dictionary containing paths for
                       `db_location` and `export_results_location`.
        outputFile (str): Path to a file to write the results to. If None, the
                          results will be printed to the console instead.

    """
    if config.get("debug") == True: print("outputFile insinde searcher : ", outputFile)
    if config.get("debug") == True: print("outputFile type : ", type(outputFile))
    tui.printLogo()
    email = str(input("Enter your email to search: "))
    tui.printLogo()
    print("Starting search...")
    print("Exporting to: ", os.path.join(str(config.get("export_results_location")) , str(outputFile)))
    for filename in os.listdir(config.get("db_location") + "/sorted"):
        # print("filename: ", filename)
        if len(email) >= 3: 
            # print(filename.endswith(".txt"))
            if filename.endswith(".txt") == True:
                stripedemail = email.strip()
                firstletter = stripedemail[0]
                secondletter = stripedemail[1]
                thirdletter = stripedemail[2]
                # print("filename: ", filename, "firstletter: ", firstletter)
                if str(filename) == str(firstletter) + str(secondletter) + str(thirdletter) + ".txt" or str(filename) == str(firstletter).upper() + str(secondletter).upper() + str(thirdletter).upper() + ".txt":
                    if config.get("debug") == True: print("looking for ", email, " in ", filename)
                    # print("Opening file: ", filename)
                    filecontent = open(config.get("db_location") + "/sorted/" + filename, "r").readlines()
                    
                    for line in filecontent:
                        if email in line:
                            if config.get("debug") == True: print("outputFile type : ", type(outputFile))
                            if config.get("debug") == True: print(outputFile , " output file")
                            if config.get("debug") == True: print(config.get("export_results_location") + " export location")
                            if config.get("debug") == True: print(os.path.join(str(config.get("export_results_location")) , outputFile) + " os.path.join")
                            if outputFile == None:
                                print(line)
                            else:
                                with open(os.path.join(str(config.get("export_results_location")) + outputFile), "a+", encoding='utf-8') as outputFile_object:
                                    outputFile_object.write(line)
                                    outputFile_object.close()
                                    if config.get("print_result_export_file") == True: print(line)
                        else: 
                            # print(line, "!=", email)
                            None

                else:
                    # print(firstletter,".txt", " != ", filename, "so", filename != str(firstletter) + ".txt")
                    # print("file not found")
                    None
            else:
                # print("file ending with .txt not found")
                None
        
        elif len(email) == 2:
            if filename.endswith(".txt") == True:
                stripedemail = email.strip()
                firstletter = stripedemail[0]
                secondletter = stripedemail[1]
                
                # print("filename: ", filename, "firstletter: ", firstletter)
                if str(filename) == str(firstletter) + str(secondletter) + ".txt":
                    # print("Opening file: ", filename)
                    filecontent = open(config.get("db_location") + "/sorted/" + filename, "r").readlines()
                    for line in filecontent:
                        if email in line:
                            if config.get("debug") == True: print(outputFile)
                            if outputFile == None:
                                print(line)
                            else:
                                with open(os.path.join(config.get("export_results_location") + outputFile), "a+", encoding='utf-8') as outputFile_object:
                                    print("Exporting to: ", config.get("export_results_location") + "output.txt")
                                    outputFile_object.write(line)
                                    outputFile_object.close()
                                    if config.get("print_result_export_file") == True: print(line)
                        else: 
                            # print(line, "!=", email)
                            None

                else:
                    # print(firstletter,".txt", " != ", filename, "so", filename != str(firstletter) + ".txt")
                    print("file not found")
                    None
            else:
                print("file ending with .txt not found")
        
        elif len(email) == 1:
            if filename.endswith(".txt") == True:
                stripedemail = email.strip()
                firstletter = stripedemail[0]

                
                # print("filename: ", filename, "firstletter: ", firstletter)
                if str(filename) == str(firstletter) + ".txt":
                    # print("Opening file: ", filename)
                    filecontent = open(config.get("db_location") + "/sorted/" + filename, "r").readlines()
                    for line in filecontent:
                        if email in line:
                            if config.get("debug") == True: print(outputFile)
                            if outputFile == None:
                                print(line)
                            else:
                                with open(os.path.join(config.get("export_results_location") + outputFile), "a+", encoding='utf-8') as outputFile_object:
                                    print("Exporting to: ", config.get("export_results_location") + "output.txt")
                                    outputFile_object.write(line)
                                    outputFile_object.close()
                                    if config.get("print_result_export_file") == True: print(line)
                        else: 
                            print(line, "!=", email)
                            None

                else:
                    # print(firstletter,".txt", " != ", filename, "so", filename != str(firstletter) + ".txt")
                    # print("file not found")
                    None
            else:
                print("file ending with .txt not found")