use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::Chars;

#[derive(PartialEq, Debug)]
pub enum Token {
    Ark,
    Mode(String),
    QuotedString(String),
    GenericString(String),
    Flag(String),
}

pub enum Mode {
    Add,
    Extract
}

impl Mode {
    pub fn new(s: String) -> Mode {
        match s.to_lowercase().as_ref() {
            "add" | "a" => Mode::Add,
            "extract" | "x" => Mode::Extract,
            _ => {
                todo!("incorrect mode specification error")
            }
        }
    }
}

// todo : better name this one sucks :(
pub struct Command {
    // whether to add the input path to an output archive,
    // or to extract the input archive into the output path
    pub mode: Option<Mode>,
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub key: Option<String>
    // todo : more options when the archiver become more advanced
}

impl Command {
    /// consume `chars` until a whitespace or end of iterator,
    /// returning the consumed characters as a `String`
    fn consume_until_whitespace(chars: &mut Chars) -> String {
        let mut res = String::new();

        loop {
            let next = chars.next();

            if next.is_none() || next.unwrap().is_whitespace() {
                break;
            }

            res.push(next.unwrap());
        }

        res
    }

    /// consume `chars` until a character matching `c` is found or
    /// end of iterator, returning the consumed characters as a `String`
    fn consume_until(chars: &mut Chars, c: char) -> String {
        let mut res = String::new();

        loop {
            let next = chars.next();

            if next.is_none() || next.unwrap() == c {
                break;
            }

            res.push(next.unwrap());
        }

        res
    }

    /// return a vector of tokens built from `S`,
    /// which represents a more generalized view of the input string
    fn tokenize(s: String) -> Vec<Token> {
        let mut chars = s.chars();
        let mut tokens: Vec<Token> = vec![];

        while chars.clone().count() > 0 {
            match chars.next() {
                // starts with '-', this is a flag
                Some('-') => {
                    tokens.push(Token::Flag(
                        Self::consume_until_whitespace(&mut chars)
                    ));
                }

                // starts with '"' this is a quoted string
                Some('"') => {
                    tokens.push(Token::QuotedString(
                        Self::consume_until(&mut chars, '"')
                    ));
                    Self::consume_until_whitespace(&mut chars);
                }

                // starts with any other char either Ark or Mode
                Some(c) => {
                    let mut res = String::new();
                    res.push(c);
                    res.push_str(&Self::consume_until_whitespace(&mut chars));

                    match res.to_lowercase().as_str() {
                        "ark" => tokens.push(Token::Ark),
                        "a" | "add" => tokens.push(Token::Mode(String::from("add"))),
                        "x" | "extract" => tokens.push(
                            Token::Mode(String::from("x"))
                        ),
                        _ => tokens.push(Token::GenericString(res))
                    }
                }
                _ => {
                    // eof, tokenization complete
                }
            }
        }
        println!("Tokens: \n\t{:?}", tokens);
        tokens
    }

    /// parse an input string into usable information for the archiver
    /// source string should be of the following format:
    /// `Ark (add/a | extract/x) "input/file/path" "output/file/path" -options...`
    pub fn new(src: String) -> Command {
        let mut toks = Self::tokenize(src)
            .into_iter()
            .rev()
            .collect::<Vec<Token>>();

        // expect Ark token to start
        if toks.pop() != Some(Token::Ark) {
            todo!("return error that command must begin with \"Ark\"")
        }

        // expect mode
        let mode = match toks.pop() {
            Some(Token::Mode(s)) => Some(Mode::new(s)),
            _ => {
                None
            }
        };

        // expect in & out paths
        let input = match toks.pop() {
            Some(Token::QuotedString(s)) => Some(PathBuf::from(s)),
            _ => None
        };

        let output = match toks.pop() {
            Some(Token::QuotedString(s)) => Some(PathBuf::from(s)),
            _ => None
        };

        let mut reader = Command {
            mode,
            input,
            output,
            key: None
        };

        // drain all remaining options
        while !toks.is_empty() {
            // guarantee that its a flag
            let flag = match toks.pop() {
                Some(Token::Flag(s)) => { s },
                _ => { todo!("improper token error") }
            };

            match flag.as_ref() {
                "-k" => {
                    todo!("get the provided key")
                }
                _ => { todo!("unknown flag error") }
            }
        }
        reader
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mode = match &self.mode {
            Some(Mode::Add) => "add",
            Some(Mode::Extract) => "extract",
            _ => "None"
        };
        let input = match &self.input {
            Some(p) => p.to_str().unwrap(),
            None => "None"
        };
        let output = match &self.output {
            Some(p) => p.to_str().unwrap(),
            None => "None"
        };

        writeln!(f, "Input:")?;
        writeln!(f, "  Mode   : {}", mode)?;
        writeln!(f, "  Input  : {}", input)?;
        writeln!(f, "  Output : {}", output)
    }
}