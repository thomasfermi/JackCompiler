//! jack_parser

use jack_tokenizer::{Token, Keyword};


use std::slice::Iter;
use std::iter::Peekable;


pub fn parse_class(tokens : &Vec<Token>) -> Result<String, &'static str> {
    println!("Hello world.");
    let mut output = "".to_string();

    let mut i = 0;

    let mut token_iterator = tokens.iter().peekable();

    if Token::Keyword(Keyword::Class) == *token_iterator.next().unwrap() {
        // class
        output += "<class>\n";
        output += "  <keyword> class </keyword>\n";

        // className
        output += &parse_name(token_iterator.next().unwrap(), 2)?;

         // {
        output += &parse_specific_symbol(token_iterator.next().unwrap(), '{', 2)?;

        // classVarDec*
        while let Some(s_class_var_dec) = parse_class_var_dec(&mut token_iterator)?{
            output += &s_class_var_dec;
        }


        // subRoutineDec*
        while let Some(s_subroutine_dec) = parse_subroutine_dec(&mut token_iterator)?{
            output += &s_subroutine_dec;
        }


        // }
        output += &parse_specific_symbol(token_iterator.next().unwrap(), '}', 2)?;


        output += "</class>";


    } else {
        return Err("This is no class!");
    }

    return Ok(output);
}

fn parse_class_var_dec(token_iterator : &mut Peekable<Iter<Token>>) -> Result<Option<String>, &'static str> {

    let mut output = "  <classVarDec>\n".to_string();

    // ( static | field )
    match token_iterator.peek().unwrap() {
        Token::Keyword(Keyword::Static) => output += "    <keyword> static </keyword>\n",
        Token::Keyword(Keyword::Field)  => output += "    <keyword> field </keyword>\n",
        _ => return Ok(None)
    }
    token_iterator.next();

    // type
    output += &parse_type(&token_iterator.next().unwrap(),4)?;

    // varName
    output += &parse_name(&token_iterator.next().unwrap(),4)?;


    // (, varName)*
    while **token_iterator.peek().unwrap() == Token::Symbol(',') {
        token_iterator.next(); // peek successful, hence next()
        output += "    <symbol> , </symbol>\n";
        output += &parse_name(token_iterator.next().unwrap(), 4)?;
    }

    // ;
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', 4)?;


    output += "  </classVarDec>\n";


    return Ok(Some(output));
}


fn parse_subroutine_dec(token_iterator : &mut Peekable<Iter<Token>>) -> Result<Option<String>, &'static str> {
    let mut output = "  <subroutineDec>\n".to_string();
    // ( constructor | function | method )
    match token_iterator.peek().unwrap() {
        Token::Keyword(Keyword::Constructor) =>  output += "    <keyword> constructor </keyword>\n",
        Token::Keyword(Keyword::Function)    =>  output += "    <keyword> function </keyword>\n",
        Token::Keyword(Keyword::Method)      =>  output += "    <keyword> method </keyword>\n",
        _ =>  return Ok(None)
    }
    token_iterator.next();

    // ( 'void' | type)
    match token_iterator.next().unwrap() {
        Token::Keyword(Keyword::Int)     =>  output += "    <keyword> int </keyword>\n",
        Token::Keyword(Keyword::Char)    =>  output += "    <keyword> char </keyword>\n",
        Token::Keyword(Keyword::Boolean) =>  output += "    <keyword> boolean </keyword>\n",
        Token::Keyword(Keyword::Void)    =>  output += "    <keyword> void </keyword>\n",
        Token::Identifier(id) =>  output += &format!("    <identifier> {} </identifier>\n",id),
        _ => return Err("Expected void or a type! Type has to be int, char, boolean, or class name!")
    }

    // subRoutineName
    if let Token::Identifier(id) = token_iterator.next().unwrap() {
        output += &format!("    <identifier> {} </identifier>\n",id);
    } else {
        return Err("Expected a subRoutine name here!");
    }

    // ( parameterList )
    output += &parse_specific_symbol(token_iterator.next().unwrap(), '(', 4)?;
    output += "    <parameterList>\n";

    if **token_iterator.peek().unwrap() != Token::Symbol(')') { // if function has more than zero arguments
        output += &parse_type(token_iterator.next().unwrap(),6)?;
        output += &parse_name(token_iterator.next().unwrap(),6)?;

        while **token_iterator.peek().unwrap() == Token::Symbol(',') {
            token_iterator.next();
            output += "      <symbol> , </symbol>\n";
            output += &parse_type(token_iterator.next().unwrap(),6)?;
            output += &parse_name(token_iterator.next().unwrap(),6)?;
        }
    }


    output += "    </parameterList>\n";
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ')', 4)?;

    // subRoutineBody
    output += "    <subroutineBody>\n";
    output += &parse_subroutine_body(token_iterator)?;
    output += "    </subroutineBody>\n";

    output += "  </subroutineDec>\n";

    return Ok(Some(output));
}

fn parse_subroutine_body(token_iterator : &mut Peekable<Iter<Token>>) -> Result<String, &'static str> {
    // {
    let mut output = parse_specific_symbol(token_iterator.next().unwrap(), '{', 6)?;
    output += "      <statements>\n";

    // varDec*
    while **token_iterator.peek().unwrap() == Token::Keyword(Keyword::Var) {
        token_iterator.next();
        output += "      <varDec>";
        output += "        <keyword> var </keyword>";
        // type
        output += &parse_type(token_iterator.next().unwrap(),8)?;

        // varName
        output += &parse_name(token_iterator.next().unwrap(), 8)?;
        // (, varName)*
        while **token_iterator.peek().unwrap() == Token::Symbol(',') {
            token_iterator.next();
            output += "    <symbol> , </symbol>\n";
            output += &parse_name(token_iterator.next().unwrap(), 8)?;
        }

        // ;
        output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', 8)?;

        output += "      </varDec>";
    }

    // statements
     while let Some(s_statement) = parse_statement(token_iterator)?{
        output += &s_statement;
     }


    output += "      </statements>\n";
    // }
    output += &parse_specific_symbol(token_iterator.next().unwrap(), '}', 6)?;

    return Ok(output);
}

fn parse_type(token : &Token, indent : usize) -> Result<String, &'static str> {
    match token {
        Token::Keyword(kw) => {
            match kw {
                Keyword::Int | Keyword::Char | Keyword::Boolean =>  Ok(format!("{:indent$}<keyword> {kw:} </keyword>\n","", indent=indent, kw=kw.to_string())),
                _ => Err("Expected a type! Type has to be int, char, boolean, or class name!"),
            }
        },
        Token::Identifier(id) =>  Ok(format!("    <identifier> {} </identifier>\n",id)),
        _ => Err("Expected a type! Type has to be int, char, boolean, or class name!")
    }
}

fn parse_name(token : &Token, indent : usize) -> Result<String, &'static str> {
    if let Token::Identifier(id) = token {
        Ok(format!("{:indent$}<identifier> {id:} </identifier>\n", "", indent=indent, id=id))
    } else {
        Err("Expected a name here!")
    }
}

fn parse_specific_symbol(token : &Token, c : char, indent : usize) -> Result<String, &'static str> {
    if *token == Token::Symbol(c) {
        Ok(format!("{:indent$}<symbol> {symbol:} </symbol>\n", "", indent=indent, symbol=c))
    }
    else {
        Err("Expected a different symbol")
    }
}

fn parse_statement(token_iterator :  &mut Peekable<Iter<Token>>) -> Result<Option<String>, &'static str> {
    let mut output = "".to_string();
    match token_iterator.peek().unwrap() {
        Token::Keyword(kw) => {
            match kw {
                Keyword::Let    =>  output += &parse_let_statement(token_iterator)?,
                Keyword::If     =>  output += &parse_if_statement(token_iterator)?,
                Keyword::While  =>  output += &parse_while_statement(token_iterator)?,
                Keyword::Do     =>  output += &parse_do_statement(token_iterator)?,
                Keyword::Return =>  output += &parse_return_statement(token_iterator)?,
                _               =>  return Err("Expected a statement beginning with let, if, while, do, or return!")
            }
        },
        _ => return Ok(None)
    }

    return Ok(Some(output));
}

fn parse_let_statement(token_iterator :  &mut Peekable<Iter<Token>>) -> Result<String, &'static str> {
    let mut output = "        <letStatement>\n".to_string();
    output        += "          <keyword> let </keyword>\n";
    token_iterator.next();

    // varName
    output += &parse_name(token_iterator.next().unwrap(), 10)?;
    // [ expression ]
    if **token_iterator.peek().unwrap() == Token::Symbol('['){
        output += &parse_specific_symbol(token_iterator.next().unwrap(),'[',10)?;
        output += &parse_expression(token_iterator, 10)?;
        output += &parse_specific_symbol(token_iterator.next().unwrap(),']',10)?;
    }

    // =
    output += &parse_specific_symbol(token_iterator.next().unwrap(), '=', 10)?;
    // expression
    output += &parse_expression(token_iterator, 10)?;
    // ;
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', 10)?;

    output    += "        </letStatement>\n";
    return Ok(output);
}

fn parse_if_statement(token_iterator :  &mut Peekable<Iter<Token>>) -> Result<String, &'static str> {
    let mut output = "        <ifStatement>\n".to_string();
    output        += "          <keyword> if </keyword>\n";
    token_iterator.next();

     // ( expression )
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'(',10)?;

    output += &parse_expression(token_iterator, 10)?;

    output += &parse_specific_symbol(token_iterator.next().unwrap(),')',10)?;

    // { statements }
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'{',10)?;

    while let Some(s_statement) = parse_statement(token_iterator)?{
        output += &s_statement;
     }

    output += &parse_specific_symbol(token_iterator.next().unwrap(),'}',10)?;


    if Token::Keyword(Keyword::Else) == **token_iterator.peek().unwrap() {
        // else
        token_iterator.next();
        output+= &format!("{:indent$}<keyword> else </keyword>\n", "", indent=10);
        // { statements }
        output += &parse_specific_symbol(token_iterator.next().unwrap(),'{',10)?;
        while let Some(s_statement) = parse_statement(token_iterator)?{
            output += &s_statement;
        }
        output += &parse_specific_symbol(token_iterator.next().unwrap(),'}',10)?;
    }

    output    += "        </ifStatement>\n";
    return Ok(output);
}

fn parse_while_statement(token_iterator :  &mut Peekable<Iter<Token>>) -> Result<String, &'static str> {
    let mut output = "        <whileStatement>\n".to_string();
    output        += "          <keyword> while </keyword>\n";
    token_iterator.next();

    // ( expression )
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'(',10)?;

    output += &parse_expression(token_iterator, 10)?;

    output += &parse_specific_symbol(token_iterator.next().unwrap(),')',10)?;

    // { statements }
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'{',10)?;

    while let Some(s_statement) = parse_statement(token_iterator)?{
        output += &s_statement;
     }

    output += &parse_specific_symbol(token_iterator.next().unwrap(),'}',10)?;

    output    += "        </whileStatement>\n";
    return Ok(output);
}

fn parse_do_statement(token_iterator :  &mut Peekable<Iter<Token>>) -> Result<String, &'static str> {
    let mut output = "        <doStatement>\n".to_string();
    output        += "          <keyword> do </keyword>\n";
    token_iterator.next();

    while **token_iterator.peek().unwrap() != Token::Symbol(';'){ //TODO: replace with real implementation
        token_iterator.next();
    }
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', 10)?;

    output    += "        </doStatement>\n";
    return Ok(output);
}

fn parse_return_statement(token_iterator :  &mut Peekable<Iter<Token>>) -> Result<String, &'static str> {
    let mut output = "        <returnStatement>\n".to_string();
    output        += "          <keyword> return </keyword>\n";
    token_iterator.next();

    if **token_iterator.peek().unwrap() != Token::Symbol(';') {
        output += &parse_expression(token_iterator, 10)?;
    }
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', 10)?;

    output    += "        </returnStatement>\n";
    return Ok(output);
}


fn parse_expression(token_iterator :  &mut Peekable<Iter<Token>>, indent : usize) -> Result<String, &'static str> {
    let mut output =  format!("{:indent$}<expression>\n", "", indent=indent);
    output += &format!("{:indent$}<term>\n", "", indent=indent+2);

    output += &parse_name(token_iterator.next().unwrap(), indent+4)?;

    output += &format!("{:indent$}</term>\n", "", indent=indent+2);
    output += &format!("{:indent$}</expression>\n", "", indent=indent);
    return Ok(output);
}
