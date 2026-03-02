use ratatui::widgets::List;

pub fn test_sorting() {
    /// The goal of this function is to sort the email.txt file in semple
    /// It should be able to store a url:logging:password as logging:password:url
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
    }
}

fn sorting_funct(login: &str) {
    let cleaned;
    let separators = [" ", ":", ",", ";"];

    if login.contains("https://") {
        // If email contains https://, then removes the https:// to prevent issues with the :
        cleaned = login.replace("https://", "");
    } else if login.contains("http://") {
        // If email contains http://, then removes the http:// to prevent issues with the :
        cleaned = login.replace("http://", "");
    } else {
        // If the login does not contains any stange formating, just add it to the db
        cleaned = login.trim().to_string();
        println!("{} is fine", cleaned);
    }
    let parts: Vec<&str> = cleaned.split(":").collect();
    // let reordered = format!("{} {} {}", parts[0], parts[1], parts[2]);
    println!("email : {}", cleaned);
    println!("split email : {:?}", parts);
}
