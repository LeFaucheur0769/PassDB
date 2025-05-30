import os

def searcher(config):
    email = str(input("Enter your email to search: "))
    script_dir = os.path.dirname(os.path.realpath(__file__))
    print("Starting search...")
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
                    print("looking for ", email, " in ", filename)
                    # print("Opening file: ", filename)
                    filecontent = open(config.get("db_location") + "/sorted/" + filename, "r").readlines()
                    for line in filecontent:
                        if email in line or email in line.upper():
                            print(line)
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
                            print(line)
                        else: 
                            print(line, "!=", email)
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
                            print(line)
                        else: 
                            print(line, "!=", email)
                            None

                else:
                    # print(firstletter,".txt", " != ", filename, "so", filename != str(firstletter) + ".txt")
                    # print("file not found")
                    None
            else:
                print("file ending with .txt not found")