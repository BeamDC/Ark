use std::io;
use std::io::BufRead;
use crate::cli::input::Command;
use crate::files::archiver::Archiver;
use crate::files::indexer::ArchiveIndexer;

mod cli;
mod files;

fn main() {
    loop {
        // read command from stdin
        let stdin = io::stdin();
        let src = stdin.lock().lines().next().unwrap().unwrap();

        // parse command
        let command = Command::new(src);
        println!("{}", &command);

        // perform specified actions
        let mut archiver = Archiver::new(command);
        archiver.operate()

        // output desired item(s)
    }
}
