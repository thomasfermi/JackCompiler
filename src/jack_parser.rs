//! jack_parser

use jack_tokenizer::Token;

pub fn parse_class(tokens : &Vec<Token>) -> String{
    println!("Hello world.");
    let mut output = "".to_string();

    let mut i = 0;


    if tokens[i] == Token::Keyword("class".to_string()) {
        // class
        output += "<class>\n";
        output += "  <keyword> class </keyword>\n";
        i+=1;


        // className
        output += &parse_name(&tokens[i], 2);
        i+=1;

        // {
        output += &parse_specific_symbol(&tokens[i], '{', 2);
        i+=1;


        // classVarDec*
        while let Some(s_class_var_dec) = parse_class_var_dec(tokens, &mut i){
            output += &s_class_var_dec;
        }

        // subRoutineDec*
        while let Some(s_subroutine_dec) = parse_subroutine_dec(tokens, &mut i){
            output += &s_subroutine_dec;
        }

        // }

        output += &parse_specific_symbol(&tokens[i], '}', 2);


        output += "</class>";

    }
    else {
        panic!("This is no class!");
    }
    output
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
    output += &parse_type(&tokens[*i],4);
    *i+=1;

    // varName
    output += &parse_name(&tokens[*i], 4);
    *i+=1;

    // (, varName)*
    while tokens[*i] == Token::Symbol(',') {
        output += "    <symbol> , </symbol>\n";
        *i += 1;
        output += &parse_name(&tokens[*i], 4);
        *i+=1;
    }

    // ;
    output += &parse_specific_symbol(&tokens[*i], ';', 4);
    *i+=1;

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
        println!("Was looking for function but found {:?}", tokens[*i]);
        return None;
    }
    *i+=1;

    // ( 'void' | type)
     match &tokens[*i] {
        Token::Keyword(kw) => {
            if kw == "int" || kw == "char" || kw == "boolean" || kw == "void" {
                output += &format!("    <keyword> {} </keyword>\n",kw);
            } else {
                panic!("Expected a type or 'void'! Type has to be int, char, boolean, or class name!")
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
    output += &parse_specific_symbol(&tokens[*i], '(', 4);
    *i+=1;
    output += "    <parameterList>\n";

    if tokens[*i] != Token::Symbol(')') { // if function has more than zero arguments
        output += &parse_type(&tokens[*i],6);
        *i+=1;
        output += &parse_name(&tokens[*i],6);
        *i+=1;
        while tokens[*i] == Token::Symbol(',') {
            output += "      <symbol> , </symbol>\n";
            *i += 1;
            output += &parse_type(&tokens[*i],6);
            *i+=1;
            output += &parse_name(&tokens[*i],6);
            *i+=1;
        }
    }


    output += "    </parameterList>\n";
    output += &parse_specific_symbol(&tokens[*i], ')', 4);
    *i+=1;


    // subRoutineBody
    output += "    <subroutineBody>\n";
    output += &parse_subroutine_body(tokens, i);
    output += "    </subroutineBody>\n";

    output += "  </subroutineDec>\n";

    return Some(output);
}

fn parse_subroutine_body(tokens : &Vec<Token>, i : &mut usize) -> String {

    // {
    let mut output = parse_specific_symbol(&tokens[*i], '{', 6);
    output += "      <statements>\n";
    *i += 1;

    // varDec*
    while tokens[*i] == Token::Keyword("var".to_string()) {
        output += "      <varDec>";
        output += "        <keyword> var </keyword>";
        *i+=1;

        // type
        output += &parse_type(&tokens[*i],8);
        *i+=1;

        // varName
        output += &parse_name(&tokens[*i], 8);
        *i+=1;

        // (, varName)*
        while tokens[*i] == Token::Symbol(',') {
            output += "    <symbol> , </symbol>\n";
            *i += 1;
            output += &parse_name(&tokens[*i], 8);
            *i+=1;
        }

        // ;
        output += &parse_specific_symbol(&tokens[*i], ';', 8);
        *i+=1;

        output += "      </varDec>";
    }

    // statements
     while let Some(s_statement) = parse_statement(tokens, i){
        output += &s_statement;
     }


    output += "      </statements>\n";
    // }
    output += &parse_specific_symbol(&tokens[*i], '}', 6);
    *i+=1;

    return output;
}

fn parse_type(token : &Token, indent : usize) -> String {
    match token {
        Token::Keyword(kw) => {
            if kw == "int" || kw == "char" || kw == "boolean" {
                format!("{:indent$}<keyword> {kw:} </keyword>\n","", indent=indent, kw=kw)
            } else {
                panic!("Expected a type! Type has to be int, char, boolean, or class name! Found token={:?}", token)
            }
        },
        Token::Identifier(id) =>  format!("    <identifier> {} </identifier>\n",id),
        _ => panic!("Expected a type! Type has to be int, char, boolean, or class name! Found token={:?}", token)
    }
}

fn parse_name(token : &Token, indent : usize) -> String {
    if let Token::Identifier(id) = token {
         format!("{:indent$}<identifier> {id:} </identifier>\n", "", indent=indent, id=id)
    } else {
        panic!("Expected a name here!");
    }
}

fn parse_specific_symbol(token : &Token, c : char, indent : usize) -> String {
    if *token == Token::Symbol(c) {
        format!("{:indent$}<symbol> {symbol:} </symbol>\n", "", indent=indent, symbol=c)
    }
    else {
        panic!("Expected {}. Got {:?}",c, token);
    }
}

fn parse_statement(tokens : &Vec<Token>, i : &mut usize) -> Option<String> {
    let mut output = "".to_string();
    match &tokens[*i] {
        Token::Keyword(kw) => {
            if kw == "let" {
                output += &parse_let_statement(tokens, i);
            } else if kw == "if" {
                output += &parse_if_statement(tokens, i);
            } else if kw == "while" {
                output += &parse_while_statement(tokens, i);
            } else if kw == "do" {
                output += &parse_do_statement(tokens, i);
            } else if kw == "return" {
                output += &parse_return_statement(tokens, i);
            } else {
                panic!("Expected a statement beginning with let, if, while, do, or return!");
            }
        },
        //_ => {println!("token={:?}",&tokens[*i]); return None},
        _ => return None
    }

    return Some(output);
}

fn parse_let_statement(tokens : &Vec<Token>, i : &mut usize) -> String {
    let mut output = "        <letStatement>\n".to_string();
    output        += "          <keyword> let </keyword>\n";
    *i+=1;

    // varName
    output += &parse_name(&tokens[*i], 10);
    *i+=1;

    // [ expression ]
    if tokens[*i] == Token::Symbol('['){
        output += &parse_specific_symbol(&tokens[*i],'[',10);
        *i+=1;

        output += &parse_expression(tokens, i, 10);

        output += &parse_specific_symbol(&tokens[*i],']',10);
        *i+=1;
    }

    // =
    output += &parse_specific_symbol(&tokens[*i], '=', 10);
    *i += 1;

    // expression
    output += &parse_expression(tokens, i, 10);

    // ;
    output += &parse_specific_symbol(&tokens[*i], ';', 10);
    *i += 1;

    output    += "        </letStatement>\n";
    return output;
}

fn parse_if_statement(tokens : &Vec<Token>, i : &mut usize) -> String {
    let mut output = "        <ifStatement>\n".to_string();
    output        += "          <keyword> if </keyword>\n";
    *i+=1;

     // ( expression )
    output += &parse_specific_symbol(&tokens[*i],'(',10);
    *i+=1;

    output += &parse_expression(tokens, i, 10);

    output += &parse_specific_symbol(&tokens[*i],')',10);
    *i+=1;

    // { statements }
    output += &parse_specific_symbol(&tokens[*i],'{',10);
    *i+=1;

    while let Some(s_statement) = parse_statement(tokens, i){
        output += &s_statement;
     }

    output += &parse_specific_symbol(&tokens[*i],'}',10);
    *i+=1;

    if let Token::Keyword(kw) = &tokens[*i] {
        if kw == "else" {
            // else
            output+= &format!("{:indent$}<keyword> else </keyword>\n", "", indent=10);
            *i+=1;
            // { statements }
            output += &parse_specific_symbol(&tokens[*i],'{',10);
            *i+=1;
            while let Some(s_statement) = parse_statement(tokens, i){
                output += &s_statement;
            }
            output += &parse_specific_symbol(&tokens[*i],'}',10);
            *i+=1;
        }
    }

    output    += "        </ifStatement>\n";
    return output;
}

fn parse_while_statement(tokens : &Vec<Token>, i : &mut usize) -> String {
    let mut output = "        <whileStatement>\n".to_string();
    output        += "          <keyword> while </keyword>\n";
    *i+=1;

    // ( expression )
    output += &parse_specific_symbol(&tokens[*i],'(',10);
    *i+=1;

    output += &parse_expression(tokens, i, 10);

    output += &parse_specific_symbol(&tokens[*i],')',10);
    *i+=1;

    // { statements }
    output += &parse_specific_symbol(&tokens[*i],'{',10);
    *i+=1;

    while let Some(s_statement) = parse_statement(tokens, i){
        output += &s_statement;
     }

    output += &parse_specific_symbol(&tokens[*i],'}',10);
    *i+=1;

    output    += "        </whileStatement>\n";
    return output;
}

fn parse_do_statement(tokens : &Vec<Token>, i : &mut usize) -> String {
    let mut output = "        <doStatement>\n".to_string();
    output        += "          <keyword> do </keyword>\n";
    *i+=1;

    while tokens[*i] != Token::Symbol(';'){ //TODO: replace with real implementation
        *i+=1;
    }
    output += &parse_specific_symbol(&tokens[*i], ';', 10);
    *i += 1;

    output    += "        </doStatement>\n";
    return output;
}

fn parse_return_statement(tokens : &Vec<Token>, i : &mut usize) -> String {
    let mut output = "        <returnStatement>\n".to_string();
    output        += "          <keyword> return </keyword>\n";
    *i+=1;

    if tokens[*i] != Token::Symbol(';') {
        output += &parse_expression(tokens, i, 10);
    }
    output += &parse_specific_symbol(&tokens[*i], ';', 10);
    *i+=1;


    output    += "        </returnStatement>\n";
    return output;
}


fn parse_expression(tokens : &Vec<Token>, i : &mut usize, indent : usize) -> String {
    let mut output =  format!("{:indent$}<expression>\n", "", indent=indent);
    output += &format!("{:indent$}<term>\n", "", indent=indent+2);

    output += &parse_name(&tokens[*i], indent+4);
    *i+=1;

    output += &format!("{:indent$}</term>\n", "", indent=indent+2);
    output += &format!("{:indent$}</expression>\n", "", indent=indent);
    return output;
}
