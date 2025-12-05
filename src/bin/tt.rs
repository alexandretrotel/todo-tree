fn main() {
    if let Err(err) = todo_tree::run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}
