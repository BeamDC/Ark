pub mod archival {
    pub mod cli {
        pub mod input;
        pub mod output;
    }

    pub mod files {
        pub mod archiver;
        pub mod header;
        pub mod indexer;
    }
}

pub fn run() {
    use std::io;
    use std::io::BufRead;
    use crate::archival::cli::input::Command;
    use crate::archival::files::archiver::{ArchivalError, Archiver};

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
            let ArchivalError(res) = archival_result.err().unwrap();
            println!("{}", res);
        }

        // output desired item(s)
    }
}


fn main() {
    run()
}