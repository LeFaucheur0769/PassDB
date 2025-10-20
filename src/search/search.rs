pub fn search() {
    println!("None");
}

struct Searcher {
    db_dir: String,
    export_dir: String,
}

impl Searcher {
    fn new(db_dir: String, export_dir: String) -> Self {
        Searcher { db_dir, export_dir }
    }
}
