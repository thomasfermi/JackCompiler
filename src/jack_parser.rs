//! jack_parser

use jack_tokenizer::Token;

pub fn parse_class(tokens : &Vec<Token>){
    println!("Hello world.");
    let mut output = "".to_string();

    let mut i = 0;


    if tokens[i] == Token::Keyword("class".to_string()) {
        // class
        output += "<class>\n";
        i+=1;
        output += "  <keyword> class </keyword>\n";

        // className
        output += &parse_name(tokens, &mut i, 2);

        // {
        output += &parse_specific_symbol(tokens, &mut i, '{', 2);


        // classVarDec*
        while let Some(s_class_var_dec) = parse_class_var_dec(tokens, &mut i){
            output += &s_class_var_dec;
        }

        // subRoutineDec*
        while let Some(s_subroutine_dec) = parse_subroutine_dec(tokens, &mut i){
            output += &s_subroutine_dec;
        }

        // }
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
    output += &parse_type(tokens, i);

    // varName
    output += &parse_name(tokens, i, 4);

    // (, varName)*
    while tokens[*i] == Token::Symbol(',') {
        output += "    <symbol> , </symbol>\n";
        *i += 1;
        output += &parse_name(tokens, i, 4);
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

    // ( 'void' | type), TODO: code reuse is bad
     match &tokens[*i] {
        Token::Keyword(kw) => {
            if kw == "int" || kw == "char" || kw == "boolean" || kw == "void" {
                output += &format!("    <keyword> {} </keyword>\n",kw);
            } else {
                panic!("Expected a type! Type has to be int, char, boolean, or class name!")
            }
        },
        Token::Identifier(id) => output += &format!("    <identifier> {} </identifier>\n",id),
        _ => panic!("Expected a type! Type has to be int, char, boolean, or class name!")
    }
    *i +=1;

    // subRoutineName, TODO: code reuse is bad
    if let Token::Identifier(id) = &tokens[*i] {
        output += &format!("    <identifier> {} </identifier>\n",id);
        *i+=1;
    } else {
        panic!("Expected a subRoutine name here!");
    }

    // ( parameterList )

    // subRoutineBody

    output += "  </subroutineDec>\n";

    return Some(output);
}

fn parse_type(tokens : &Vec<Token>, i : &mut usize) -> String {
    match &tokens[*i] {
        Token::Keyword(kw) => {
            if kw == "int" || kw == "char" || kw == "boolean" {
                *i +=1;
                format!("    <keyword> {} </keyword>\n",kw)
            } else {
                panic!("Expected a type! Type has to be int, char, boolean, or class name!")
            }
        },
        Token::Identifier(id) => {
            *i+=1;
            format!("    <identifier> {} </identifier>\n",id)
        },
        _ => panic!("Expected a type! Type has to be int, char, boolean, or class name!")
    }
}

fn parse_name(tokens : &Vec<Token>, i : &mut usize, indent : usize) -> String {
    if let Token::Identifier(id) = &tokens[*i] {
         *i+=1;
         format!("{:indent$}<identifier> {id:} </identifier>\n", "", indent=indent, id=id)
    } else {
        panic!("Expected a name here!");
    }
}

fn parse_specific_symbol(tokens : &Vec<Token>, i : &mut usize, c : char, indent : usize) -> String {
    if tokens[*i] == Token::Symbol('{') {
        *i+=1;
        format!("{:indent$}<symbol> {symbol:} </symbol>\n", "", indent=indent, symbol=c)
    }
    else {
        panic!("Expected {");
    }
}