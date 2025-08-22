use std::io;
use std::io::BufRead;
use crate::cli::input::InputReader;
use crate::files::indexer::ArchiveIndexer;

mod cli;
mod files;

fn main() {
    loop {
        // read command from stdin
        let stdin = io::stdin();
        let src = stdin.lock().lines().next().unwrap().unwrap();

        // parse command
        let command = InputReader::new(src);
        println!("{}", command);

        // perform specified actions
        let mut indexer = ArchiveIndexer::new(command.input.unwrap());
        indexer.index_files();

        // output desired item(s)
    }
}
