//! jack_compiler

use jack_tokenizer::{Keyword, Token};

use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone)]
enum JackVariableType {
    Jint,
    Jchar,
    Jboolean,
    Jclass(String),
}

#[derive(Debug, Clone)]
enum JackClassVariableKind {
    Jstatic(usize),
    Jfield(usize),
}

#[derive(Debug, Clone)]
enum JackMethodVariableKind{
    Jargument(usize),
    Jvar(usize),
}

#[derive(Debug, Clone)]
struct JackClassSymbolTableEntry{
    var_name : String, 
    var_type : JackVariableType,
    var_kind : JackClassVariableKind,
}

#[derive(Debug, Clone)]
struct JackClassSymbolTable { //TODO: Should be hash table, so that variables can be fetched quickly. 
    entries : Vec<JackClassSymbolTableEntry>,
    num_fields : usize,
    num_statics : usize,
}

#[derive(Debug, Clone)]
struct JackMethodSymbolTable {
    entries : Vec<JackMethodSymbolTableEntry>,
    num_args : usize,
    num_vars : usize,
}

#[derive(Debug, Clone)]
struct JackMethodSymbolTableEntry{
    var_name : String,
    var_type : JackVariableType,
    var_kind : JackMethodVariableKind,
}

impl JackMethodSymbolTableEntry {
    fn new(var_name:String, var_type:JackVariableType, var_kind:JackMethodVariableKind) -> Self {
        JackMethodSymbolTableEntry {
            var_name,
            var_type,
            var_kind,
        }
    }
}


/// JackCompiler struct
pub struct JackCompiler<'a> {
    token_iterator: Peekable<Iter<'a, Token>>,
    vm_output: String,
    class_name : String,
    class_symbol_table : JackClassSymbolTable,
    method_symbol_table : JackMethodSymbolTable,
}

impl<'a> JackCompiler<'a> {
    /// Constructor
    pub fn new(tokens: &'a [Token]) -> Self {
        JackCompiler {
            token_iterator: tokens.iter().peekable(),
            vm_output: "".to_string(),
            class_name: "".to_string(),
            class_symbol_table : JackClassSymbolTable{entries:vec![], num_fields:0, num_statics:0},
            method_symbol_table : JackMethodSymbolTable{entries:vec![], num_args:0, num_vars:0},
        }
    }
    /// Main function
    pub fn parse_class(&mut self) -> Result<String, &'static str> {
        if Token::Keyword(Keyword::Class) == *self.token_iterator.next().unwrap() {
            // class
            self.vm_output += "<class>\n";
            self.vm_output += "  <keyword> class </keyword>\n";

            // className
            self.class_name = format!("{}",self.parse_name(2)?);

            // {
            self.parse_specific_symbol('{', 2)?;

            // classVarDec*
            while self.parse_class_var_dec()? {
                // do nothing
            }
            println!("table={:?}", self.class_symbol_table);

            // subRoutineDec*
            while self.parse_subroutine_dec()? {
                println!("table={:?}", self.method_symbol_table);
                // do nothing
            }

            // }
            self.parse_specific_symbol('}', 2)?;

            self.vm_output += "</class>";
        } else {
            return Err("This is no class!");
        }

        return Ok(self.vm_output.clone());
    }

    fn parse_class_var_dec(&mut self) -> Result<bool, &'static str> {
        // ( static | field )
        let mut var_kind = match self.token_iterator.peek().unwrap() {
            Token::Keyword(Keyword::Static) => {
                self.vm_output += "  <classVarDec>\n    <keyword> static </keyword>\n"; 
                self.class_symbol_table.num_statics+=1;               
                JackClassVariableKind::Jstatic(self.class_symbol_table.num_statics-1)
            }
            Token::Keyword(Keyword::Field) => {
                self.vm_output += "  <classVarDec>\n    <keyword> field </keyword>\n";
                self.class_symbol_table.num_fields+=1;
                JackClassVariableKind::Jfield(self.class_symbol_table.num_fields-1)              
            }
            _ => return Ok(false),
        };
        self.token_iterator.next();

        // type
        let var_type = self.parse_type(4)?;

        // varName
        let mut var_name = format!("{}",self.parse_name(4)?);

        self.class_symbol_table.entries.push(
            JackClassSymbolTableEntry{
               var_name,
               var_type : var_type.clone(),
               var_kind : var_kind.clone(),                
            }
        );

        // (, varName)*
        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.token_iterator.next(); // peek successful, hence next()
            self.vm_output += "    <symbol> , </symbol>\n";
            
            var_kind = match var_kind {
                JackClassVariableKind::Jstatic(_num) => {
                    self.class_symbol_table.num_statics+=1;
                    JackClassVariableKind::Jstatic(self.class_symbol_table.num_statics-1)
                }
                JackClassVariableKind::Jfield(_num) => {
                    self.class_symbol_table.num_fields+=1;
                    JackClassVariableKind::Jfield(self.class_symbol_table.num_fields-1)
                }
            };
            var_name = format!("{}",self.parse_name(4)?);

            self.class_symbol_table.entries.push(
                JackClassSymbolTableEntry{
                    var_name,
                    var_type : var_type.clone(),
                    var_kind : var_kind.clone(),                
                }
            );
        }

        // ;
        self.parse_specific_symbol(';', 4)?;

        self.vm_output += "  </classVarDec>\n";

        return Ok(true);
    }

    fn parse_subroutine_dec(&mut self) -> Result<bool, &'static str> {
        // forget about last symbol table from last function and initialize new one
        self.method_symbol_table = JackMethodSymbolTable{entries:vec![], num_args:0, num_vars:0};

        // ( constructor | function | method )
        match self.token_iterator.peek().unwrap() {
            Token::Keyword(Keyword::Constructor) => {
                self.vm_output += "  <subroutineDec>\n    <keyword> constructor </keyword>\n"
            }
            Token::Keyword(Keyword::Function) => {
                self.vm_output += "  <subroutineDec>\n    <keyword> function </keyword>\n"
            }
            Token::Keyword(Keyword::Method) => {
                self.vm_output += "  <subroutineDec>\n    <keyword> method </keyword>\n"
            }
            _ => return Ok(false),
        }
        self.token_iterator.next();

        // ( 'void' | type)
        match self.token_iterator.next().unwrap() {
            Token::Keyword(Keyword::Int) => self.vm_output += "    <keyword> int </keyword>\n",
            Token::Keyword(Keyword::Char) => self.vm_output += "    <keyword> char </keyword>\n",
            Token::Keyword(Keyword::Boolean) => {
                self.vm_output += "    <keyword> boolean </keyword>\n"
            }
            Token::Keyword(Keyword::Void) => self.vm_output += "    <keyword> void </keyword>\n",
            Token::Identifier(id) => {
                self.vm_output += &format!("    <identifier> {} </identifier>\n", id)
            }
            _ => {
                return Err(
                    "Expected void or a type! Type has to be int, char, boolean, or class name!",
                )
            }
        }

        // subRoutineName
        if let Token::Identifier(id) = self.token_iterator.next().unwrap() {
            self.vm_output += &format!("    <identifier> {} </identifier>\n", id);
        } else {
            return Err("Expected a subRoutine name here!");
        }

        // ( parameterList )
        self.parse_specific_symbol('(', 4)?;
        self.vm_output += "    <parameterList>\n";

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') { // if function has more than zero arguments
            let var_type = self.parse_type(6)?;
            let var_name = format!("{}",self.parse_name(6)?);
            self.method_symbol_table.entries.push(
                    JackMethodSymbolTableEntry::new(var_name, var_type, JackMethodVariableKind::Jargument(0))
            );
            self.method_symbol_table.num_args=1;

            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                self.vm_output += "      <symbol> , </symbol>\n";
                let var_type = self.parse_type(6)?;
                let var_name = format!("{}",self.parse_name(6)?);
                self.method_symbol_table.entries.push(
                    JackMethodSymbolTableEntry::new(var_name, var_type, JackMethodVariableKind::Jargument(self.method_symbol_table.num_args))  
                );
                self.method_symbol_table.num_args+=1;
            }
        }

        self.vm_output += "    </parameterList>\n";
        self.parse_specific_symbol(')', 4)?;

        // subRoutineBody
        self.vm_output += "    <subroutineBody>\n";
        self.parse_subroutine_body()?;
        self.vm_output += "    </subroutineBody>\n";

        self.vm_output += "  </subroutineDec>\n";

        return Ok(true);
    }

    fn parse_subroutine_body(&mut self) -> Result<(), &'static str> {
        // {
        self.parse_specific_symbol('{', 6)?;

        // varDec*
        while **self.token_iterator.peek().unwrap() == Token::Keyword(Keyword::Var) {
            self.token_iterator.next();
            self.vm_output += "      <varDec>\n";
            self.vm_output += "        <keyword> var </keyword>\n";
            // type
            let var_type = self.parse_type(8)?;

            // varName
            let var_name = format!("{}",self.parse_name(8)?);

            
            self.method_symbol_table.entries.push(
                JackMethodSymbolTableEntry::new(var_name,var_type.clone(),JackMethodVariableKind::Jvar(self.method_symbol_table.num_vars))
            );
            self.method_symbol_table.num_vars+=1;

            // (, varName)*
            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                self.vm_output += "        <symbol> , </symbol>\n";
                let var_name = format!("{}",self.parse_name(8)?);

                self.method_symbol_table.entries.push(
                    JackMethodSymbolTableEntry::new(var_name,var_type.clone(),JackMethodVariableKind::Jvar(self.method_symbol_table.num_vars))
                );
                self.method_symbol_table.num_vars+=1;
            }

            // ;
            self.parse_specific_symbol(';', 8)?;

            self.vm_output += "      </varDec>\n";
        }

        // statements
        self.vm_output += "      <statements>\n";
        while self.parse_statement(8)? {
            // do nothing
        }
        self.vm_output += "      </statements>\n";

        // }
        self.parse_specific_symbol('}', 6)?;

        return Ok(());
    }

    fn parse_statement(&mut self, xml_indent: usize) -> Result<bool, &'static str> {
        match self.token_iterator.peek().unwrap() {
            Token::Keyword(kw) => match kw {
                Keyword::Let => self.parse_let_statement(xml_indent)?,
                Keyword::If => self.parse_if_statement(xml_indent)?,
                Keyword::While => self.parse_while_statement(xml_indent)?,
                Keyword::Do => self.parse_do_statement(xml_indent)?,
                Keyword::Return => self.parse_return_statement(xml_indent)?,
                _ => {
                    return Err("Expected a statement beginning with let, if, while, do, or return!")
                }
            },
            _ => return Ok(false),
        }
        return Ok(true);
    }

    fn parse_let_statement(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<letStatement>\n", "", indent = xml_indent);
        self.vm_output += &format!(
            "{:indent$}<keyword> let </keyword>\n",
            "",
            indent = xml_indent + 2
        );
        self.token_iterator.next();

        // varName
        self.parse_name(xml_indent + 2)?;
        // [ expression ]
        if **self.token_iterator.peek().unwrap() == Token::Symbol('[') {
            self.parse_specific_symbol('[', xml_indent + 2)?;
            self.parse_expression(xml_indent + 2)?;
            self.parse_specific_symbol(']', xml_indent + 2)?;
        }

        // =
        self.parse_specific_symbol('=', xml_indent + 2)?;
        // expression
        self.parse_expression(xml_indent + 2)?;
        // ;
        self.parse_specific_symbol(';', xml_indent + 2)?;

        self.vm_output += &format!("{:indent$}</letStatement>\n", "", indent = xml_indent);
        return Ok(());
    }

    fn parse_if_statement(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<ifStatement>\n", "", indent = xml_indent);
        self.vm_output += &format!(
            "{:indent$}<keyword> if </keyword>\n",
            "",
            indent = xml_indent + 2
        );
        self.token_iterator.next();

        // ( expression )
        self.parse_specific_symbol('(', xml_indent + 2)?;

        self.parse_expression(xml_indent + 2)?;

        self.parse_specific_symbol(')', xml_indent + 2)?;

        // { statements }
        self.parse_specific_symbol('{', xml_indent + 2)?;

        self.vm_output += &format!("{:indent$}<statements>\n", "", indent = xml_indent + 2);
        while self.parse_statement(xml_indent + 4)? {
            //do nothing
        }
        self.vm_output += &format!("{:indent$}</statements>\n", "", indent = xml_indent + 2);

        self.parse_specific_symbol('}', xml_indent + 2)?;

        if Token::Keyword(Keyword::Else) == **self.token_iterator.peek().unwrap() {
            // else
            self.token_iterator.next();
            self.vm_output += &format!(
                "{:indent$}<keyword> else </keyword>\n",
                "",
                indent = xml_indent + 2
            );
            // { statements }
            self.parse_specific_symbol('{', xml_indent + 2)?;
            self.vm_output += &format!("{:indent$}<statements>\n", "", indent = xml_indent + 2);
            while self.parse_statement(xml_indent + 4)? {
                // do nothing
            }
            self.vm_output += &format!("{:indent$}</statements>\n", "", indent = xml_indent + 2);
            self.parse_specific_symbol('}', xml_indent + 2)?;
        }

        self.vm_output += &format!("{:indent$}</ifStatement>\n", "", indent = xml_indent);
        return Ok(());
    }

    fn parse_while_statement(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<whileStatement>\n", "", indent = xml_indent);
        self.vm_output += &format!(
            "{:indent$}<keyword> while </keyword>\n",
            "",
            indent = xml_indent + 2
        );
        self.token_iterator.next();

        // ( expression )
        self.parse_specific_symbol('(', xml_indent + 2)?;

        self.parse_expression(xml_indent + 2)?;

        self.parse_specific_symbol(')', xml_indent + 2)?;

        // { statements }
        self.parse_specific_symbol('{', xml_indent + 2)?;
        self.vm_output += &format!("{:indent$}<statements>\n", "", indent = xml_indent + 2);
        while self.parse_statement(xml_indent + 4)? {
            // do nothing
        }
        self.vm_output += &format!("{:indent$}</statements>\n", "", indent = xml_indent + 2);

        self.parse_specific_symbol('}', xml_indent + 2)?;

        self.vm_output += &format!("{:indent$}</whileStatement>\n", "", indent = xml_indent);
        return Ok(());
    }

    fn parse_do_statement(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<doStatement>\n", "", indent = xml_indent);
        self.vm_output += &format!(
            "{:indent$}<keyword> do </keyword>\n",
            "",
            indent = xml_indent + 2
        );
        self.token_iterator.next();

        self.parse_subroutine_call(xml_indent + 2)?;

        while **self.token_iterator.peek().unwrap() != Token::Symbol(';') {
            //TODO: replace with real implementation
            self.token_iterator.next();
        }
        self.parse_specific_symbol(';', xml_indent + 2)?;

        self.vm_output += &format!("{:indent$}</doStatement>\n", "", indent = xml_indent);
        return Ok(());
    }

    fn parse_return_statement(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<returnStatement>\n", "", indent = xml_indent);
        self.vm_output += &format!(
            "{:indent$}<keyword> return </keyword>\n",
            "",
            indent = xml_indent + 2
        );
        self.token_iterator.next();

        if **self.token_iterator.peek().unwrap() != Token::Symbol(';') {
            self.parse_expression(xml_indent + 2)?;
        }
        self.parse_specific_symbol(';', xml_indent + 2)?;

        self.vm_output += &format!("{:indent$}</returnStatement>\n", "", indent = xml_indent);
        return Ok(());
    }

    fn parse_expression(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<expression>\n", "", indent = xml_indent);
        self.parse_term(xml_indent + 2)?;
        while self.find_operation(xml_indent + 2)? {
            self.parse_term(xml_indent + 2)?;
        }

        self.vm_output += &format!("{:indent$}</expression>\n", "", indent = xml_indent);
        return Ok(());
    }

    fn parse_term(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += &format!("{:indent$}<term>\n", "", indent = xml_indent);

        match self.token_iterator.peek().unwrap() {
            Token::IntConstant(i) => {
                self.vm_output += &format!(
                    "{:indent$}<integerConstant> {i:} </integerConstant>\n",
                    "",
                    indent = xml_indent + 2,
                    i = i
                );
                self.token_iterator.next();
            }
            Token::StringConstant(s) => {
                self.vm_output += &format!(
                    "{:indent$}<stringConstant> {s:} </stringConstant>\n",
                    "",
                    indent = xml_indent + 2,
                    s = s
                );
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::True) => {
                self.vm_output += &format!(
                    "{:indent$}<keyword> true </keyword>\n",
                    "",
                    indent = xml_indent + 2
                );
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::False) => {
                self.vm_output += &format!(
                    "{:indent$}<keyword> false </keyword>\n",
                    "",
                    indent = xml_indent + 2
                );
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::Null) => {
                self.vm_output += &format!(
                    "{:indent$}<keyword> null </keyword>\n",
                    "",
                    indent = xml_indent + 2
                );
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::This) => {
                self.vm_output += &format!(
                    "{:indent$}<keyword> this </keyword>\n",
                    "",
                    indent = xml_indent + 2
                );
                self.token_iterator.next();
            }
            // (expression)
            Token::Symbol('(') => {
                self.parse_specific_symbol('(', xml_indent + 2)?;
                self.parse_expression(xml_indent + 2)?;
                self.parse_specific_symbol(')', xml_indent + 2)?;
            }
            // unaryOp term
            Token::Symbol('-') => {
                self.parse_specific_symbol('-', xml_indent + 2)?;
                self.parse_term(xml_indent + 2)?;
            }
            Token::Symbol('~') => {
                self.parse_specific_symbol('~', xml_indent + 2)?;
                self.parse_term(xml_indent + 2)?;
            }
            // varname | varname[expression] | subroutineCall
            //TODO: subroutine call handling should be done via the parse_subroutine function ideally :/
            Token::Identifier(name) => {
                self.vm_output += &format!(
                    "{:indent$}<identifier> {name} </identifier>\n",
                    "",
                    indent = xml_indent + 2,
                    name = name
                );
                self.token_iterator.next();
                match **self.token_iterator.peek().unwrap() {
                    // varName[expression]
                    Token::Symbol('[') => {
                        self.parse_specific_symbol('[', xml_indent + 2)?;
                        self.parse_expression(xml_indent + 2)?;
                        self.parse_specific_symbol(']', xml_indent + 2)?;
                    }
                    // var_name.function_name()
                    Token::Symbol('.') => {
                        self.parse_specific_symbol('.', xml_indent + 2)?;
                        self.parse_name(xml_indent + 2)?;
                        self.parse_expression_list(xml_indent + 2)?;
                    }
                    // function_name()
                    Token::Symbol('(') => {
                        self.parse_expression_list(xml_indent + 2)?;
                    }
                    // simply the var_name
                    _ => {}
                }
            }
            Token::Symbol(_s) => return Err("This symbol is not a term"),
            _ => return Err("This token is not a term"),
        }

        self.vm_output += &format!("{:indent$}</term>\n", "", indent = xml_indent);

        return Ok(());
    }

    fn parse_expression_list(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        // (
        self.parse_specific_symbol('(', xml_indent)?;
        self.vm_output += &format!("{:indent$}<expressionList>\n", "", indent = xml_indent);

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') {
            self.parse_expression(xml_indent + 2)?;
        }

        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.parse_specific_symbol(',', xml_indent + 2)?;
            self.parse_expression(xml_indent + 2)?;
        }

        self.vm_output += &format!("{:indent$}</expressionList>\n", "", indent = xml_indent);
        // )
        self.parse_specific_symbol(')', xml_indent)?;
        return Ok(());
    }

    fn parse_subroutine_call(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.parse_name(xml_indent)?;
        // if a dot follows, we have the case className|varName . subRoutineName, otherwise it is just subroutineName
        if **self.token_iterator.peek().unwrap() == Token::Symbol('.') {
            self.parse_specific_symbol('.', xml_indent)?;
            self.parse_name(xml_indent)?;
        }
        self.parse_expression_list(xml_indent)?;

        return Ok(());
    }

    fn find_operation(&mut self, xml_indent: usize) -> Result<bool, &'static str> {
        match self.token_iterator.peek().unwrap() {
            Token::Symbol('+') => self.parse_specific_symbol('+', xml_indent)?,
            Token::Symbol('-') => self.parse_specific_symbol('-', xml_indent)?,
            Token::Symbol('*') => self.parse_specific_symbol('*', xml_indent)?,
            Token::Symbol('/') => self.parse_specific_symbol('/', xml_indent)?,
            Token::Symbol('&') => self.parse_specific_symbol('&', xml_indent)?,
            Token::Symbol('|') => self.parse_specific_symbol('|', xml_indent)?,
            Token::Symbol('<') => self.parse_specific_symbol('<', xml_indent)?,
            Token::Symbol('>') => self.parse_specific_symbol('>', xml_indent)?,
            Token::Symbol('=') => self.parse_specific_symbol('=', xml_indent)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn parse_type(&mut self, xml_indent: usize) -> Result<JackVariableType, &'static str> {
        match self.token_iterator.next().unwrap() {
            Token::Identifier(identifier) => {
                self.vm_output += &format!(
                    "{:indent$}<identifier> {id:} </identifier>\n",
                    "",
                    indent = xml_indent,
                    id = identifier
                );
                Ok(JackVariableType::Jclass(identifier.to_string()))
            },
            Token::Keyword(Keyword::Int)=> {
                self.vm_output += &format!(
                    "{:indent$}<keyword> int </keyword>\n",
                    "",
                    indent = xml_indent
                );
                Ok(JackVariableType::Jint)
            },
            Token::Keyword(Keyword::Char)=> {
                self.vm_output += &format!(
                    "{:indent$}<keyword> char </keyword>\n",
                    "",
                    indent = xml_indent
                );
                Ok(JackVariableType::Jchar)
            },
            Token::Keyword(Keyword::Boolean)=> {
                self.vm_output += &format!(
                    "{:indent$}<keyword> boolean </keyword>\n",
                    "",
                    indent = xml_indent
                );
                Ok(JackVariableType::Jboolean)
            },
            _ => Err("Expected a type! Type has to be int, char, boolean, or class name!"),
        }
    }

    fn parse_name(&mut self, xml_indent: usize) -> Result<&str, &'static str> {
        if let Token::Identifier(id) = self.token_iterator.next().unwrap() {
            self.vm_output += &format!(
                "{:indent$}<identifier> {id:} </identifier>\n",
                "",
                indent = xml_indent,
                id = id
            );
            Ok(id)
        } else {
            Err("Expected a name here!")
        }
    }

    fn parse_specific_symbol(&mut self, c: char, xml_indent: usize) -> Result<(), &'static str> {
        if *self.token_iterator.next().unwrap() == Token::Symbol(c) {
            let c_xml = match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                _ => c.to_string(),
            };
            self.vm_output += &format!(
                "{:indent$}<symbol> {symbol:} </symbol>\n",
                "",
                indent = xml_indent,
                symbol = c_xml
            );
            Ok(())
        } else {
            Err("Expected a different symbol")
        }
    }
}
