//! jack_compiler

use jack_tokenizer::{Token, Keyword};


use std::slice::Iter;
use std::iter::Peekable;

/// JackCompiler struct
pub struct JackCompiler<'a> {
    token_iterator : Peekable<Iter<'a, Token>>,
    vm_output : String, 
}


impl<'a> JackCompiler<'a> {
    /// Constructor
    pub fn new(tokens : &'a[Token]) -> Self{
        JackCompiler{
            token_iterator : tokens.iter().peekable(),
            vm_output : "".to_string(),
        }
    }
    /// Main function
    pub fn parse_class(&mut self) -> Result<String, &'static str> {
        
        
        if Token::Keyword(Keyword::Class) == *self.token_iterator.next().unwrap() {
            // class
            self.vm_output += "<class>\n";
            self.vm_output += "  <keyword> class </keyword>\n";

            // className
            self.vm_output += &parse_name(self.token_iterator.next().unwrap(), 2)?;

            // {
            self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), '{', 2)?;

            // classVarDec*
            while self.parse_class_var_dec()?{
                // do nothing
            }


            // subRoutineDec*
            while self.parse_subroutine_dec()?{
                // do nothing
            }


            // }
            self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), '}', 2)?;


            self.vm_output += "</class>";


        } else {
            return Err("This is no class!");
        }

        return Ok(self.vm_output.clone());
    }


    fn parse_class_var_dec(&mut self) -> Result<bool, &'static str> {

        // ( static | field )
        match self.token_iterator.peek().unwrap() {
            Token::Keyword(Keyword::Static) => self.vm_output += "  <classVarDec>\n    <keyword> static </keyword>\n",
            Token::Keyword(Keyword::Field)  => self.vm_output += "  <classVarDec>\n    <keyword> field </keyword>\n",
            _ => return Ok(false)
        }
        self.token_iterator.next();

        // type
        self.vm_output += &parse_type(&self.token_iterator.next().unwrap(),4)?;

        // varName
        self.vm_output += &parse_name(&self.token_iterator.next().unwrap(),4)?;


        // (, varName)*
        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.token_iterator.next(); // peek successful, hence next()
            self.vm_output += "    <symbol> , </symbol>\n";
            self.vm_output += &parse_name(self.token_iterator.next().unwrap(), 4)?;
        }

        // ;
        self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), ';', 4)?;


        self.vm_output += "  </classVarDec>\n";


        return Ok(true);
    }

    fn parse_subroutine_dec(&mut self) -> Result<bool, &'static str>  {
        // ( constructor | function | method )
        match self.token_iterator.peek().unwrap() {
            Token::Keyword(Keyword::Constructor) =>  self.vm_output += "  <subroutineDec>\n    <keyword> constructor </keyword>\n",
            Token::Keyword(Keyword::Function)    =>  self.vm_output += "  <subroutineDec>\n    <keyword> function </keyword>\n",
            Token::Keyword(Keyword::Method)      =>  self.vm_output += "  <subroutineDec>\n    <keyword> method </keyword>\n",
            _ =>  return Ok(false)
        }
        self.token_iterator.next();

        // ( 'void' | type)
        match self.token_iterator.next().unwrap() {
            Token::Keyword(Keyword::Int)     =>  self.vm_output += "    <keyword> int </keyword>\n",
            Token::Keyword(Keyword::Char)    =>  self.vm_output += "    <keyword> char </keyword>\n",
            Token::Keyword(Keyword::Boolean) =>  self.vm_output += "    <keyword> boolean </keyword>\n",
            Token::Keyword(Keyword::Void)    =>  self.vm_output += "    <keyword> void </keyword>\n",
            Token::Identifier(id) =>  self.vm_output += &format!("    <identifier> {} </identifier>\n",id),
            _ => return Err("Expected void or a type! Type has to be int, char, boolean, or class name!")
        }

        // subRoutineName
        if let Token::Identifier(id) = self.token_iterator.next().unwrap() {
            self.vm_output += &format!("    <identifier> {} </identifier>\n",id);
        } else {
            return Err("Expected a subRoutine name here!");
        }

        // ( parameterList )
        self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), '(', 4)?;
        self.vm_output += "    <parameterList>\n";

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') { // if function has more than zero arguments
            self.vm_output += &parse_type(self.token_iterator.next().unwrap(),6)?;
            self.vm_output += &parse_name(self.token_iterator.next().unwrap(),6)?;

            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                self.vm_output += "      <symbol> , </symbol>\n";
                self.vm_output += &parse_type(self.token_iterator.next().unwrap(),6)?;
                self.vm_output += &parse_name(self.token_iterator.next().unwrap(),6)?;
            }
        }


        self.vm_output += "    </parameterList>\n";
        self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), ')', 4)?;

        // subRoutineBody
        self.vm_output += "    <subroutineBody>\n";
        self.parse_subroutine_body()?;
        self.vm_output += "    </subroutineBody>\n";

        self.vm_output += "  </subroutineDec>\n";

        return Ok(true);
    }

    fn parse_subroutine_body(&mut self) -> Result<(), &'static str> {
        // {
        self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), '{', 6)?;

        // varDec*
        while **self.token_iterator.peek().unwrap() == Token::Keyword(Keyword::Var) {
            self.token_iterator.next();
            self.vm_output += "      <varDec>\n";
            self.vm_output += "        <keyword> var </keyword>\n";
            // type
            self.vm_output += &parse_type(self.token_iterator.next().unwrap(),8)?;

            // varName
            self.vm_output += &parse_name(self.token_iterator.next().unwrap(), 8)?;
            // (, varName)*
            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                self.vm_output += "        <symbol> , </symbol>\n";
                self.vm_output += &parse_name(self.token_iterator.next().unwrap(), 8)?;
            }

            // ;
            self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), ';', 8)?;

            self.vm_output += "      </varDec>\n";
        }

        // statements
        self.vm_output += "      <statements>\n";
        while self.parse_statement(8)?{
            // do nothing
        }
        self.vm_output += "      </statements>\n";

        // }
        self.vm_output += &parse_specific_symbol(self.token_iterator.next().unwrap(), '}', 6)?;

        return Ok(());
    }



    fn parse_statement(&mut self, xml_indent : usize) -> Result<bool, &'static str> {
        
        match self.token_iterator.peek().unwrap() {
            Token::Keyword(kw) => {
                match kw {
                    Keyword::Let    =>  self.vm_output += &parse_let_statement(&mut self.token_iterator, xml_indent)?,
                    Keyword::If     =>  self.vm_output += &parse_if_statement(&mut self.token_iterator, xml_indent)?,
                    Keyword::While  =>  self.vm_output += &parse_while_statement(&mut self.token_iterator, xml_indent)?,
                    Keyword::Do     =>  self.vm_output += &parse_do_statement(&mut self.token_iterator, xml_indent)?,
                    Keyword::Return =>  self.vm_output += &parse_return_statement(&mut self.token_iterator, xml_indent)?,
                    _               =>  return Err("Expected a statement beginning with let, if, while, do, or return!")
                }
            },
            _ => return Ok(false)
        }
        return Ok(true);
    }

}




fn parse_type(token : &Token, xml_indent : usize) -> Result<String, &'static str> {
    match token {
        Token::Keyword(kw) => {
            match kw {
                Keyword::Int | Keyword::Char | Keyword::Boolean =>  Ok(format!("{:indent$}<keyword> {kw:} </keyword>\n","", indent=xml_indent, kw=kw.to_string())),
                _ => Err("Expected a type! Type has to be int, char, boolean, or class name!"),
            }
        },
        Token::Identifier(id) =>  Ok(format!("{:indent$}<identifier> {id:} </identifier>\n","",indent=xml_indent,id=id)),
        _ => Err("Expected a type! Type has to be int, char, boolean, or class name!")
    }
}

fn parse_name(token : &Token, xml_indent : usize) -> Result<String, &'static str> {
    if let Token::Identifier(id) = token {
        Ok(format!("{:indent$}<identifier> {id:} </identifier>\n", "", indent=xml_indent, id=id))
    } else {
        Err("Expected a name here!")
    }
}

fn parse_specific_symbol(token : &Token, c : char, xml_indent : usize) -> Result<String, &'static str> {
    if *token == Token::Symbol(c) {
        let c_xml = match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            _   => c.to_string(),
        };
        Ok(format!("{:indent$}<symbol> {symbol:} </symbol>\n", "", indent=xml_indent, symbol=c_xml))
    }
    else {
        Err("Expected a different symbol")
    }
}




fn parse_let_statement(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {

    let mut output = format!("{:indent$}<letStatement>\n", "", indent=xml_indent);
    output += &format!("{:indent$}<keyword> let </keyword>\n", "", indent=xml_indent+2);
    token_iterator.next();

    // varName
    output += &parse_name(token_iterator.next().unwrap(), xml_indent+2)?;
    // [ expression ]
    if **token_iterator.peek().unwrap() == Token::Symbol('['){
        output += &parse_specific_symbol(token_iterator.next().unwrap(),'[', xml_indent+2)?;
        output += &parse_expression(token_iterator, xml_indent+2)?;
        output += &parse_specific_symbol(token_iterator.next().unwrap(),']', xml_indent+2)?;
    }

    // =
    output += &parse_specific_symbol(token_iterator.next().unwrap(), '=', xml_indent+2)?;
    // expression
    output += &parse_expression(token_iterator, xml_indent+2)?;
    // ;
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', xml_indent+2)?;

    output    += &format!("{:indent$}</letStatement>\n", "", indent=xml_indent);
    return Ok(output);
}


fn parse_if_statement(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    let mut output = format!("{:indent$}<ifStatement>\n", "", indent=xml_indent);
    output        += &format!("{:indent$}<keyword> if </keyword>\n", "", indent=xml_indent+2);
    token_iterator.next();

     // ( expression )
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'(',xml_indent+2)?;

    output += &parse_expression(token_iterator, xml_indent + 2)?;

    output += &parse_specific_symbol(token_iterator.next().unwrap(),')',xml_indent + 2)?;

    // { statements }
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'{',xml_indent + 2)?;

    output += &format!("{:indent$}<statements>\n", "", indent=xml_indent+2);
    while let Some(s_statement) = parse_statement(token_iterator, xml_indent + 4)?{
        output += &s_statement;
     }
    output += &format!("{:indent$}</statements>\n", "", indent=xml_indent+2);


    output += &parse_specific_symbol(token_iterator.next().unwrap(),'}',xml_indent + 2)?;


    if Token::Keyword(Keyword::Else) == **token_iterator.peek().unwrap() {
        // else
        token_iterator.next();
        output+= &format!("{:indent$}<keyword> else </keyword>\n", "", indent=xml_indent + 2);
        // { statements }
        output += &parse_specific_symbol(token_iterator.next().unwrap(),'{',xml_indent + 2)?;
        output += &format!("{:indent$}<statements>\n", "", indent=xml_indent+2);
        while let Some(s_statement) = parse_statement(token_iterator, xml_indent + 4)?{
            output += &s_statement;
        }
        output += &format!("{:indent$}</statements>\n", "", indent=xml_indent+2);
        output += &parse_specific_symbol(token_iterator.next().unwrap(),'}',xml_indent + 2)?;
    }

    output    += &format!("{:indent$}</ifStatement>\n", "", indent=xml_indent);
    return Ok(output);
}


fn parse_while_statement(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    let mut output = format!("{:indent$}<whileStatement>\n", "", indent=xml_indent);
    output        += &format!("{:indent$}<keyword> while </keyword>\n", "", indent=xml_indent+2);
    token_iterator.next();

    // ( expression )
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'(',xml_indent + 2)?;

    output += &parse_expression(token_iterator, xml_indent + 2)?;

    output += &parse_specific_symbol(token_iterator.next().unwrap(),')',xml_indent + 2)?;

    // { statements }
    output += &parse_specific_symbol(token_iterator.next().unwrap(),'{',xml_indent + 2)?;
    output += &format!("{:indent$}<statements>\n", "", indent=xml_indent+2);
    while let Some(s_statement) = parse_statement(token_iterator, xml_indent+4)?{
        output += &s_statement;
     }
    output += &format!("{:indent$}</statements>\n", "", indent=xml_indent+2);

    output += &parse_specific_symbol(token_iterator.next().unwrap(),'}',xml_indent + 2)?;

    output    += &format!("{:indent$}</whileStatement>\n", "", indent=xml_indent);
    return Ok(output);
}


fn parse_do_statement(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    let mut output = format!("{:indent$}<doStatement>\n", "", indent=xml_indent);
    output        += &format!("{:indent$}<keyword> do </keyword>\n", "", indent=xml_indent+2);
    token_iterator.next();

    output += &parse_subroutine_call(token_iterator, xml_indent+2)?;

    while **token_iterator.peek().unwrap() != Token::Symbol(';'){ //TODO: replace with real implementation
        token_iterator.next();
    }
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', xml_indent+2)?;

    output    += &format!("{:indent$}</doStatement>\n", "", indent=xml_indent);
    return Ok(output);
}

fn parse_subroutine_call(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str>{
    let mut output = parse_name(token_iterator.next().unwrap(), xml_indent)?;
    // if a dot follows, we have the case className|varName . subRoutineName, otherwise it is just subroutineName
    if **token_iterator.peek().unwrap() == Token::Symbol('.') {
        output += &parse_specific_symbol(token_iterator.next().unwrap(), '.', xml_indent)?;
        output += &parse_name(token_iterator.next().unwrap(), xml_indent)?;
    }
    output += &parse_expression_list(token_iterator, xml_indent)?;

    return Ok(output);
}


fn parse_return_statement(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    let mut output = format!("{:indent$}<returnStatement>\n", "", indent=xml_indent);
    output        += &format!("{:indent$}<keyword> return </keyword>\n", "", indent=xml_indent+2);
    token_iterator.next();

    if **token_iterator.peek().unwrap() != Token::Symbol(';') {
        output += &parse_expression(token_iterator, xml_indent+2)?;
    }
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ';', xml_indent+2)?;

    output    += &format!("{:indent$}</returnStatement>\n", "", indent=xml_indent);
    return Ok(output);
}

fn parse_expression_list(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    // (
    let mut output = parse_specific_symbol(token_iterator.next().unwrap(), '(', xml_indent)?;
    output += &format!("{:indent$}<expressionList>\n", "", indent=xml_indent);

    if **token_iterator.peek().unwrap() != Token::Symbol(')') {
        output += &parse_expression(token_iterator, xml_indent+2)?;
    }

    while **token_iterator.peek().unwrap() == Token::Symbol(','){
        output += &parse_specific_symbol(token_iterator.next().unwrap(), ',', xml_indent+2)?;
        output += &parse_expression(token_iterator, xml_indent+2)?;
    }

    output += &format!("{:indent$}</expressionList>\n", "", indent=xml_indent);
    // )
    output += &parse_specific_symbol(token_iterator.next().unwrap(), ')', xml_indent)?;
    return Ok(output);
}


fn parse_expression(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    let mut output =  format!("{:indent$}<expression>\n", "", indent=xml_indent);
    output += &parse_term(token_iterator, xml_indent+2)?;
    while let Some(s_op) = peek_operation(token_iterator, xml_indent+2)?{
        token_iterator.next();
        output += &s_op;
        output += &parse_term(token_iterator, xml_indent+2)?;
     }

    output += &format!("{:indent$}</expression>\n", "", indent=xml_indent);
    return Ok(output);
}

fn parse_term(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<String, &'static str> {
    let mut output = format!("{:indent$}<term>\n", "", indent=xml_indent);

    //output += &parse_name(token_iterator.next().unwrap(), xml_indent+4)?;

    match token_iterator.peek().unwrap() {
        Token::IntConstant(i)           => {
            output += &format!("{:indent$}<integerConstant> {i:} </integerConstant>\n", "", indent=xml_indent+2, i=i);
            token_iterator.next();
        },
        Token::StringConstant(s)        => {
            output += &format!("{:indent$}<stringConstant> {s:} </stringConstant>\n", "", indent=xml_indent+2, s=s);
            token_iterator.next();
        },
        Token::Keyword(Keyword::True)   => {
            output += &format!("{:indent$}<keyword> true </keyword>\n", "", indent=xml_indent+2);
            token_iterator.next();
        },
        Token::Keyword(Keyword::False)  => {
            output += &format!("{:indent$}<keyword> false </keyword>\n", "", indent=xml_indent+2);
            token_iterator.next();
        },
        Token::Keyword(Keyword::Null)   => {
            output += &format!("{:indent$}<keyword> null </keyword>\n", "", indent=xml_indent+2);
            token_iterator.next();
        },       
        Token::Keyword(Keyword::This)   => {
            output += &format!("{:indent$}<keyword> this </keyword>\n", "", indent=xml_indent+2);
            token_iterator.next();
        },
        // (expression)
        Token::Symbol('(')              =>   {
            output += &parse_specific_symbol(token_iterator.next().unwrap(), '(', xml_indent+2)?;
            output += &parse_expression(token_iterator, xml_indent+2)?;
            output += &parse_specific_symbol(token_iterator.next().unwrap(), ')', xml_indent+2)?;
        },
        // unaryOp term
        Token::Symbol('-')              => {
            output += &parse_specific_symbol(token_iterator.next().unwrap(), '-', xml_indent+2)?;
            output += &parse_term(token_iterator, xml_indent+2)?;
        },
         Token::Symbol('~')              => {
            output += &parse_specific_symbol(token_iterator.next().unwrap(), '~', xml_indent+2)?;
            output += &parse_term(token_iterator, xml_indent+2)?;
        },
        // varname | varname[expression] | subroutineCall
        //TODO: subroutine call handling should be done via the parse_subroutine function ideally :/
        Token::Identifier(name)      => {
            output += &format!("{:indent$}<identifier> {name} </identifier>\n", "", indent=xml_indent+2, name=name);
            token_iterator.next();
            match **token_iterator.peek().unwrap() {
                // varName[expression]
                Token::Symbol('[') => {
                    output += &parse_specific_symbol(token_iterator.next().unwrap(), '[', xml_indent+2)?;
                    output += &parse_expression(token_iterator, xml_indent+2)?;
                    output += &parse_specific_symbol(token_iterator.next().unwrap(), ']', xml_indent+2)?;
                },
                // var_name.function_name()
                Token::Symbol('.') => {
                    output += &parse_specific_symbol(token_iterator.next().unwrap(), '.', xml_indent+2)?;
                    output += &parse_name(token_iterator.next().unwrap(), xml_indent+2)?;
                    output += &parse_expression_list(token_iterator, xml_indent+2)?;
                }, 
                // function_name()
                Token::Symbol('(') => {
                    output += &parse_expression_list(token_iterator, xml_indent+2)?;
                },
                // simply the var_name
                _ => {}
            }
        },
        Token::Symbol(s) => return Err("This symbol is not a term"),
        _ => return Err("This token is not a term")
    }


    output += &format!("{:indent$}</term>\n", "", indent=xml_indent);

    return Ok(output);
}

fn peek_operation(token_iterator :  &mut Peekable<Iter<Token>>, xml_indent : usize) -> Result<Option<String>, &'static str> {
    match token_iterator.peek().unwrap() {
        Token::Symbol('+') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'+', xml_indent)?)),
        Token::Symbol('-') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'-', xml_indent)?)),
        Token::Symbol('*') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'*', xml_indent)?)),
        Token::Symbol('/') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'/', xml_indent)?)),
        Token::Symbol('&') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'&', xml_indent)?)),
        Token::Symbol('|') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'|', xml_indent)?)),
        Token::Symbol('<') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'<', xml_indent)?)),
        Token::Symbol('>') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'>', xml_indent)?)),
        Token::Symbol('=') => Ok(Some(parse_specific_symbol(token_iterator.peek().unwrap(),'=', xml_indent)?)),
        _ => Ok(None)
    }
}

