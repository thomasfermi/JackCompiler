//! jack_parser

use jack_tokenizer::Token;

pub fn parse_class(tokens : &Vec<Token>){
    println!("Hello world.");
    let mut output = "".to_string();

    let mut i = 0;


    if tokens[i] == Token::Keyword("class".to_string()) {
        output += "<class>\n";
        i+=1;
        output += "  <keyword> class </keyword>\n";

        if let Token::Identifier(s) = &tokens[i] {
             output += &format!("  <identifier> {} </identifier>\n", s);
        }
        else {
            panic!("This is not good :(");
        }
        i+=1;

        if tokens[i] == Token::Symbol('{') {
            output += "  <symbol> { </symbol>\n";
        }
        else {
            panic!("This is not good :(");
        }
        i+=1;

        // parse as many class variable declarations as you can
        while let Some(s_class_var_dec) = parse_class_var_dec(tokens, &mut i){
            output += &s_class_var_dec;
        }

        // parse as many subroutine declarations as you can
        while let Some(s_subroutine_dec) = parse_subroutine_dec(tokens, &mut i){
            output += &s_subroutine_dec;
        }

        /*
        if tokens[i] == Token::Symbol('}') {
            output += "  <symbol> } </symbol>\n";
        }
        else {
            panic!("This is not good :(");
        }
        */

        output += "</class>";

    }
    else {
        panic!("This is no class!");
    }
    println!("{}", output);
}

fn parse_class_var_dec(tokens : &Vec<Token>, i : &mut usize) -> Option<String> {
    let mut output = "  <classVarDec>\n".to_string();

    // ( static | field )
    if tokens[*i] == Token::Keyword("static".to_string()) {
        output += "    <keyword> static </keyword>\n";
    } else if tokens[*i] == Token::Keyword("field".to_string()) {
        output += "    <keyword> field </keyword>\n";
    } else {
        return None;
    }
    *i+=1;

    // type
    match &tokens[*i] {
        Token::Keyword(kw) => {
            if kw == "int" || kw == "char" || kw == "boolean" {
                output += &format!("    <keyword> {} </keyword>\n",kw);
            } else {
                panic!("Expected a type! Type has to be int, char, boolean, or class name!")
            }
        },
        Token::Identifier(id) => output += &format!("    <identifier> {} </identifier>\n",id),
        _ => panic!("Expected a type! Type has to be int, char, boolean, or class name!")
    }
    *i +=1;

    // varName
    if let Token::Identifier(id) = &tokens[*i] {
        output += &format!("    <identifier> {} </identifier>\n",id);
        *i+=1;
    } else {
        panic!("Expected a variable name here!");
    }

    // (, varName)*
    while tokens[*i] == Token::Symbol(',') {
        output += "    <symbol> , </symbol>\n";
        *i += 1;
        if let Token::Identifier(id) = &tokens[*i] {
            output += &format!("    <identifier> {} </identifier>\n",id);
            *i+=1;
        } else {
            panic!("Expected a variable name here!");
        }
    }

    // ;
    if tokens[*i] == Token::Symbol(';') {
        output += "    <symbol> ; </symbol>\n";
        *i += 1;
    }
    else {
        panic!("Expected a ';'");
    }

    output += "  </classVarDec>\n";


    return Some(output);
}

fn parse_subroutine_dec(tokens : &Vec<Token>, i : &mut usize) -> Option<String> {
    let mut output = "  <subroutineDec>\n".to_string();

    // ( constructor | function | method )
    if tokens[*i] == Token::Keyword("constructor".to_string()) {
        output += "    <keyword> constructor </keyword>\n";
    } else if tokens[*i] == Token::Keyword("function".to_string()) {
        output += "    <keyword> function </keyword>\n";
    } else if tokens[*i] == Token::Keyword("method".to_string()) {
        output += "    <keyword> method </keyword>\n";
    } else {
        return None;
    }
    *i+=1;

    //

    output += "  </subroutineDec>\n";

    return Some(output);
}