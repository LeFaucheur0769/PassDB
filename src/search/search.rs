use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

pub fn search() {
    println!("None");
}

struct Searcher {
    db_dir: String,
    export_dir: String,
    file: File,
    output: Vec<String>,
    email_to_search: String,
    nbr_line: u64,
}

impl Searcher {
    fn new(db_dir: String, export_dir: String, email_to_search: String) -> Self {
        let mut first_3_letters = email_to_search.clone();
        first_3_letters.truncate(3);
        Searcher {
            db_dir,
            export_dir,
            file: File::open(format!("{}.txt", first_3_letters)).unwrap(),
            output: vec![],
            email_to_search,
            nbr_line: 0,
        }
    }

    fn search(&mut self) {}

    fn update(&mut self) -> std::io::Result<bool> {
        let reader = BufReader::new(&self.file):;
        let nbr_line = reader.lines().count();
        self.nbr_line = nbr_line as u64;
        for line in reader.lines() {
            if line.as_ref().unwrap().contains(&self.email_to_search) {
                self.output.push(line.unwrap());
            }
        }

        Ok(true)
    }
}
