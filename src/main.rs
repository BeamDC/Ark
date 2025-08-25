use std::io;
use std::io::BufRead;
use crate::cli::input::Command;
use crate::files::archiver::Archiver;

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
        let archival_result = archiver.operate();
        if archival_result.is_err() {
            panic!("archiver encountered an error")
        }

        // output desired item(s)
    }
}
