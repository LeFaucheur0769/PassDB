use crossterm::style::Stylize;
use ratatui::widgets::List;

/// The goal of this function is to sort the email.txt file in semple
/// It should be able to store a url:logging:password as logging:password:url
pub fn test_sorting() {
    let email = [
        "https://test.com:username:password",
        "username:password",
        "email@test.com:password",
        "username password",
        "email@test.com password",
        "username,password",
        "username:password;otherfield;age;test;location;other"
    ];
    for login in email.iter() {
        sorting_funct(login);
        // check_for_separators(login, false);
        // reorder_ulp_to_lpu(login, &':');
    }
}

fn sorting_funct(login: &str) -> String {
    let mut cleaned;
    let separators = [" ", ":", ",", ";"];

    // Check if contains url parts and if yes clean them
    if check_if_contains_url(login) {
        cleaned = clean_url_in_login(login);
    } else {
        cleaned = login.to_string()
    }

    separator_function_tmp_name(cleaned.as_str())

}

/// Check if the login contains any url sign and if yes, return true
fn check_if_contains_url(login: &str) -> bool {
    let url_parts_to_check = ["https://", "http://", "www."];
    for part_to_check in url_parts_to_check {
        if login.contains(part_to_check) {
            return true;
        }
    }
    false
}

/// Removes any kind of url specific characters that could cause issues with the process of
/// adding it to the db
fn clean_url_in_login(login: &str) -> String {
    let mut cleaned: String;

    // removes the https:// to prevent issues with the :
    cleaned = login.replace("https://", "");

    //  removes the http:// to prevent issues with the :
    cleaned = cleaned.replace("http://", "");

    // If the login does not contain any strange formating, just add it to the db
    cleaned = cleaned.trim().to_string();

    cleaned
}

/// Temporary separator function not optimized with a lot of code in it
/// Needs to be torn down soon and split
fn separator_function_tmp_name(login: &str) -> String {
    // The available separators
    let list_separators = [':', ';', ',', ' '];
    let mut valid_separators: Vec<(char, u8)> = vec![];
    let mut output: String = "".to_string();

    // Check for separators
    for separator in list_separators {
        let separator_appearance = login.chars().filter(|x| *x == separator).count();
        // Return invalid format if no separators
        if separator_appearance == 0 {
            // println!("{} invalid format for '{}'", login, separator)
        }
        // Todo if the format is a valid lp
        if separator_appearance == 1 {
            // Add the separator and the number of appearance
            valid_separators.push((separator, separator_appearance as u8));
        }
        // Todo if the format is a valid ulp
        if separator_appearance == 2 {
            valid_separators.push((separator, separator_appearance as u8));
        }

        // Return invalid format if more than 2 separators
        // Might have to do something for the stealer logs
        if separator_appearance > 2 {
            // println!("{} invalid format for '{}'", login, separator)
        }
    }

    // Check if multiple separators and add the number to a vec of u8
    let appears_once_or_twice: Vec<u8> = valid_separators
        .iter()
        .filter(|(_, value)| *value == 1 || *value == 2)
        .map(|(_, value)| *value)
        .collect();

    // Check if only one separator appears once or twice
    if appears_once_or_twice.iter().count() != 1 {
        if valid_separators
            .iter()
            .filter(|(_, value)| *value == 1 || *value == 2)
            .count()
            == 1
            && valid_separators
                .iter()
                .filter(|(_, value)| *value > 2)
                .count()
                == 1
        {
            /*
            Todo
                Considers that there is a strange thing but separators are still found so the line is good to import
                return login
            */
            output = login.to_string();


        }
    } else {
        // There is only one separator that appears once or twice
        if appears_once_or_twice.iter().any(|&x| x == 2) {
            /*
            Todo
                Considered as ulp
            */
            for (sep, c) in valid_separators {
                if c == 2 {
                    let split = login.split(sep).collect::<Vec<&str>>();
                    if split.len() == 3 {
                        /*
                      Considers that the format of the import is ulp and so moving it to lpu
                      */
                            return(format!("{}:{}:{}", split[1], split[2], split[0]))

                    } else {

                        // println!("{} invalid format for '{}'", login, sep);
                    }

                }
            }
        } else {
            /*
            Todo
                considered as valid combo, good to import
            */
            return login.to_string()
        }
    }
    output
    // split at the separator if doable
}

///
fn reorder_ulp_to_lpu(login: &str, separator: &char) {
    let split_login = login.split(":").collect::<Vec<&str>>();
    println!("split login : {:?}", split_login);
    return;
}
