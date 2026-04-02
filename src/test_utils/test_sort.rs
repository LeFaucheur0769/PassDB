use crossterm::style::Stylize;

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
    ];
    for login in email.iter() {
        sorting_funct(login);
        // check_for_separators(login, false);
        // reorder_ulp_to_lpu(login, &':');
    }
}

fn sorting_funct(login: &str) {
    let mut cleaned;
    let separators = [" ", ":", ",", ";"];

    // Check if contains url parts and if yes clean them
    if check_if_contains_url(login) {
        cleaned = clean_url_in_login(login);
    } else { cleaned = login.to_string() }

    check_if_contains_url(cleaned.as_str());




   /* if login.contains("https://") {
        // If email contains https://, then removes the https:// to prevent issues with the :
        cleaned = login.replace("https://", "");
    } else if login.contains("http://") {
        // If email contains http://, then removes the http:// to prevent issues with the :
        cleaned = login.replace("http://", "");
    } else {
        // If the login does not contain any strange formating, just add it to the db
        cleaned = login.trim().to_string();
        println!("{} is fine", cleaned);
    }*/
    /*let parts: Vec<&str> = cleaned.split(":").collect();*/
    // let reordered = format!("{} {} {}", parts[0], parts[1], parts[2]);


    println!("email : {}", cleaned);
    // println!("split email : {:?}", parts);
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

/// Check if the login contains separators and if not, return invalid format
/// Return 0 if no separator and the separator
/// Return 1 if format is lp (one separator) and the separator
/// Return 2 if format is ulp (two separators) and the separator
fn check_for_separators(login: &str) {
    // The available separators
    let list_separators = [":", ";", ",", " "];

    // Check for separators
    for separator in list_separators {
        let separator_appearance = login.chars().filter(|x| x.to_string() == separator).count();
        // Return invalid format if no separators
        if separator_appearance == 0 {
            println!("{} invalid format for '{}'", login, separator)
        }
        // Todo if the format is a valid lp
        if separator_appearance == 1 {
            todo!()
        }
        // Todo if the format is a valid ulp
        if separator_appearance == 2 {
            todo!()
        }

        // Return invalid format if more than 2 separators
        // Might have to do something for the stealer logs
        if separator_appearance > 2 {
            println!("{} invalid format for '{}'", login, separator)
        }
    }
}

///
fn reorder_ulp_to_lpu(login: &str, separator: &char) {
    let split_login = login.split(":").collect::<Vec<&str>>();
    println!("split login : {:?}", split_login);
    return;
}
