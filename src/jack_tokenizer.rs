//! jack_tokenizer

extern crate regex;
use self::regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Symbol(char),
    Identifier(String),
    IntConstant(i32),
    StringConstant(String),
}

/// Consumes a string that is the content of a *.jack program and converts it to tokenized xml file
pub fn tokenize(mut jack_code : String) -> Vec<Token> {
    // remove comments that are done with //
    let mut re = Regex::new(r"//.*\n").unwrap();
    jack_code = re.replace_all(&jack_code, "").into_owned();

    //remove comments that are done with /* ... */
    // (?s) is a flag that changes behavior of "." in regex. "." will match new lines as well
    re = Regex::new(r"/\*(?s).*?\*/").unwrap(); // .*? is .* but non-greedy
    jack_code = re.replace_all(&jack_code, "").into_owned();



    let mut tokens = vec![];
    let mut current_string = "".to_string();
    let list_containing_whitespace_and_newline_and_all_symbols = vec![
        ' ', '\n', '\r', '\t', '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/',
        '&', '|', '<', '>', '=', '~',
    ];


    let mut inside_string_literal = false;

    let mut chars = jack_code.chars();
    while let Some(c) = chars.next() {

        if inside_string_literal {
            if c == '\"' { // string literal ends
                tokens.push(Token::StringConstant(current_string.clone()));
                current_string = "".to_string();
                inside_string_literal = false;
            } else {
                current_string.push(c);
            }
        }
        else if c=='\"' { //string literal begins
            inside_string_literal = true;
        }
        else if list_containing_whitespace_and_newline_and_all_symbols.contains(&c) {
            // tokenize string that came before this symbol
            if !current_string.is_empty() {
                tokens.push(tokenize_single_string(&current_string));
                current_string = "".to_string();
            }
            // tokenize current symbols
            if c != ' ' && c != '\n' && c != '\r' && c != '\t' {
                tokens.push(Token::Symbol(c))
            }
        } else {
            current_string.push(c);
        }
    }

    tokens

}

pub fn tokens_to_xml(tokens : Vec<Token>) -> String {
    let mut output = "<tokens>\n".to_string();

    for t in tokens {
        match t {
            Token::Keyword(keyword) => output += &format!("<keyword> {} </keyword>\n",keyword),
            Token::Symbol(symbol) => {
                let modified_symbol : String = match symbol {
                    '<' => "&lt;".to_string(),
                    '>' => "&gt;".to_string(),
                    '&' => "&amp;".to_string(),
                    _ => symbol.to_string(),
                };
                output += &format!("<symbol> {} </symbol>\n",modified_symbol);
            },
            Token::Identifier(identifier) => output += &format!("<identifier> {} </identifier>\n",identifier),
            Token::IntConstant(int_constant) => output += &format!("<integerConstant> {} </integerConstant>\n",int_constant),
            Token::StringConstant(string_constant) => output += &format!("<stringConstant> {} </stringConstant>\n",string_constant),
        }
    }

    output += "</tokens>";

    return output;
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
    else if let Ok(num) = s.parse::<i32>() {
        Token::IntConstant(num)

    } else{
        // TODO: check if s is a sequence of letters digits and underscores not starting with a digit
        Token::Identifier(s.to_string())
    }


}
