//! jack_tokenizer

extern crate regex;
use self::regex::Regex;
use std::str::Lines;

#[derive(Debug)]
pub enum Token {
    Keyword(String),
    Symbol(char),
    Identifier(String),
    IntConstant(usize),
    StringConstant(String),
}

/// JackTokenizer struct
pub struct JackTokenizer {
    /// string containing jack code
    jack_code: String,
    /// The current line of the assembly code file is copied into a string
    current_token: Option<String>,
}

impl JackTokenizer {
    /// Creates new JackTokenizer
    pub fn new(jack_code: String) -> Self {
        JackTokenizer {
            jack_code,
            current_token: None,
        }
    }

    /// main function
    pub fn tokenize(&mut self) -> String {
        // remove comments that are done with //
        let mut re = Regex::new(r"//.*\n").unwrap();
        self.jack_code = re.replace_all(&self.jack_code, "").into_owned();

        //remove comments that are done with /* ... */
        // (?s) is a flag that changes behavior of "." in regex. "." will match new lines as well
        re = Regex::new(r"/\*(?s).*\*/").unwrap();
        self.jack_code = re.replace_all(&self.jack_code, "").into_owned();

        let mut chars = self.jack_code.chars();
        let c = chars.next();

        let mut tokens = vec![];
        let mut current_string = "".to_string();
        let list_containing_whitespace_and_newline_and_all_symbols = vec![
            ' ', '\n', '\r', '\t', '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/',
            '&', '|', '<', '>', '=', '~',
        ];

        //TODO: check whether we are inside a string literal

        while let Some(c) = chars.next() {
            if list_containing_whitespace_and_newline_and_all_symbols.contains(&c) {
                // tokenize string that came before this symbol
                if !current_string.is_empty() {
                    tokens.push(tokenize_single_string(&current_string));
                    current_string = "".to_string();
                }
                // tokenize current symbole
                if c != ' ' && c != '\n' && c != '\r' && c != '\t' {
                    tokens.push(Token::Symbol(c))
                }
            } else {
                current_string.push(c);
            }
        }

        for t in tokens {
            println!("{:?}", t);
        }

        return self.jack_code.clone();
    }
}

pub fn tokenize_single_string(s: &String) -> Token {
    let keywords = vec![
        "class",
        "constructor",
        "function",
        "method",
        "field",
        "static",
        "var",
        "int",
        "char",
        "boolean",
        "void",
        "true",
        "false",
        "null",
        "this",
        "let",
        "do",
        "if",
        "else",
        "while",
        "return",
    ];


    if keywords.contains(&&**s) {
        Token::Keyword(s.to_string())
    }
    else {
        //TODO: could also be integer constant at this point
        Token::Identifier(s.to_string())
    }


}
