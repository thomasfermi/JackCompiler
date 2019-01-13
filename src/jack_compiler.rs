//! jack_compiler

use jack_tokenizer::{Keyword, Token};

use std::collections::HashMap;
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
struct SymbolTableEntry {
    var_type: JackVariableType,
    num: usize,
}

impl SymbolTableEntry {
    fn new(var_type: JackVariableType, num: usize) -> Self {
        SymbolTableEntry { var_type, num }
    }
}

#[derive(Debug, Clone)]
enum VariableKind {
    Jstatic,
    Jfield,
    Jvar,
    Jarg,
}

#[derive(Debug, Clone)]
enum FunctionKind {
    Jmethod,
    Jconstructor,
    Jfunction,
}

#[derive(Debug, Clone)]
enum JackOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    Less,
    Larger,
    Equal,
}

impl JackOperation {
    fn to_vm_command_string(&self) -> String {
        match self {
            JackOperation::Add => "add\n".to_string(),
            JackOperation::Subtract => "sub\n".to_string(),
            JackOperation::Multiply => "call Math.multiply 2\n".to_string(),
            JackOperation::Divide => "call Math.divide 2\n".to_string(),
            JackOperation::And => "and\n".to_string(),
            JackOperation::Or => "or\n".to_string(),
            JackOperation::Less => "lt\n".to_string(),
            JackOperation::Larger => "gt\n".to_string(),
            JackOperation::Equal => "eq\n".to_string()
        }
    }
}

/// JackCompiler struct
pub struct JackCompiler<'a> {
    token_iterator: Peekable<Iter<'a, Token>>,
    vm_output: String,
    class_name: String,
    field_symbol_table: HashMap<String, SymbolTableEntry>,
    static_symbol_table: HashMap<String, SymbolTableEntry>,
    var_symbol_table: HashMap<String, SymbolTableEntry>,
    arg_symbol_table: HashMap<String, SymbolTableEntry>,
}

impl<'a> JackCompiler<'a> {
    /// Constructor
    pub fn new(tokens: &'a [Token]) -> Self {
        JackCompiler {
            token_iterator: tokens.iter().peekable(),
            vm_output: "".to_string(),
            class_name: "".to_string(),
            field_symbol_table: HashMap::new(),
            static_symbol_table: HashMap::new(),
            var_symbol_table: HashMap::new(),
            arg_symbol_table: HashMap::new(),
        }
    }

    fn add_to_symbol_table(
        &mut self,
        var_type: JackVariableType,
        var_kind: VariableKind,
        var_name: String,
    ) -> Result<(), &'static str> {
        match var_kind {
            VariableKind::Jstatic => {
                if !self.static_symbol_table.contains_key(&var_name)
                    && !self.field_symbol_table.contains_key(&var_name)
                {
                    let len = self.static_symbol_table.len();
                    self.static_symbol_table
                        .insert(var_name, SymbolTableEntry::new(var_type, len));
                } else {
                    return Err("This variable name is already in use!");
                }
            }
            VariableKind::Jfield => {
                if !self.static_symbol_table.contains_key(&var_name)
                    && !self.field_symbol_table.contains_key(&var_name)
                {
                    let len = self.field_symbol_table.len();
                    self.field_symbol_table
                        .insert(var_name, SymbolTableEntry::new(var_type, len));
                } else {
                    return Err("This variable name is already in use!");
                }
            }
            VariableKind::Jvar => {
                if !self.var_symbol_table.contains_key(&var_name)
                    && !self.arg_symbol_table.contains_key(&var_name)
                {
                    let len = self.var_symbol_table.len();
                    self.var_symbol_table
                        .insert(var_name, SymbolTableEntry::new(var_type, len));
                } else {
                    return Err("This variable name is already in use!");
                }
            }
            VariableKind::Jarg => {
                if !self.var_symbol_table.contains_key(&var_name)
                    && !self.arg_symbol_table.contains_key(&var_name)
                {
                    let len = self.arg_symbol_table.len();
                    self.arg_symbol_table
                        .insert(var_name, SymbolTableEntry::new(var_type, len));
                } else {
                    return Err("This variable name is already in use!");
                }
            }
        }
        Ok(())
    }

    /// Main function
    /// TODO: Write custom Err structs and use them instead of static str
    /// https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html
    pub fn parse_class(&mut self) -> Result<String, &'static str> {
        if Token::Keyword(Keyword::Class) == *self.token_iterator.next().unwrap() {
            // className
            self.class_name = self.parse_name(2)?.to_owned();
            self.parse_specific_symbol('{', 2)?;

            // classVarDec*
            while self.parse_class_var_dec()? {
                // do nothing
            }
            println!("field_table={:?}", self.field_symbol_table);
            println!("static_table={:?}", self.static_symbol_table);

            // subRoutineDec*
            while self.parse_subroutine_dec()? {
                println!("var_table={:?}", self.var_symbol_table);
                println!("arg_table={:?}", self.arg_symbol_table);
                // do nothing
            }

            // }
            self.parse_specific_symbol('}', 2)?;

        } else {
            return Err("This is no class!");
        }

        return Ok(self.vm_output.clone());
    }

    fn parse_class_var_dec(&mut self) -> Result<bool, &'static str> {
        // ( static | field )
        let var_kind = match self.token_iterator.peek().unwrap() {
            Token::Keyword(Keyword::Static) => {
                VariableKind::Jstatic
            }
            Token::Keyword(Keyword::Field) => {
                VariableKind::Jfield
            }
            _ => return Ok(false),
        };
        self.token_iterator.next();

        // type
        let var_type = self.parse_type(4)?;

        // varName
        let mut var_name = self.parse_name(4)?.to_owned();

        self.add_to_symbol_table(var_type.clone(), var_kind.clone(), var_name)?;

        // (, varName)*
        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.token_iterator.next(); // peek successful, hence next()

            var_name = self.parse_name(4)?.to_owned();

            self.add_to_symbol_table(var_type.clone(), var_kind.clone(), var_name)?;
        }

        // ;
        self.parse_specific_symbol(';', 4)?;

        return Ok(true);
    }

    fn parse_subroutine_dec(&mut self) -> Result<bool, &'static str> {
        // forget about last symbol table from last function and initialize new one
        self.arg_symbol_table = HashMap::new();

        // ( constructor | function | method )
        let function_kind = match self.token_iterator.peek().unwrap() {
            Token::Keyword(Keyword::Constructor) => {
                FunctionKind::Jconstructor
            }
            Token::Keyword(Keyword::Function) => {
                FunctionKind::Jfunction
            }
            Token::Keyword(Keyword::Method) => {
                let class_name = self.class_name.clone();
                self.add_to_symbol_table(
                    JackVariableType::Jclass(class_name),
                    VariableKind::Jarg,
                    "this".to_string(),
                )?;
                FunctionKind::Jmethod
            }
            _ => return Ok(false),
        };
        self.token_iterator.next();

        // ( 'void' | type)
        let return_type = match self.token_iterator.next().unwrap() {
            Token::Keyword(Keyword::Int)     => Some(JackVariableType::Jint),
            Token::Keyword(Keyword::Char)    => Some(JackVariableType::Jchar),
            Token::Keyword(Keyword::Boolean) => Some(JackVariableType::Jboolean),
            Token::Keyword(Keyword::Void)    => None,
            Token::Identifier(id)            => Some(JackVariableType::Jclass(id.to_owned())),
            _ => {
                return Err(
                    "Expected void or a type! Type has to be int, char, boolean, or class name!",
                )
            }
        };

        // subRoutineName
        let fname = match self.token_iterator.next().unwrap() {
            Token::Identifier(name) => name,
            _ => return Err("Expected a subRoutine name here!")
        };


        self.vm_output += &format!("function {class_name:}.{fname:} ", class_name=self.class_name, fname=fname);

        // ( parameterList )
        self.parse_specific_symbol('(', 4)?;

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') {
            // if function has more than zero arguments
            let var_type = self.parse_type(6)?;
            let var_name = format!("{}", self.parse_name(6)?);
            self.add_to_symbol_table(var_type, VariableKind::Jarg, var_name)?;

            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                self.vm_output += "      <symbol> , </symbol>\n";
                let var_type = self.parse_type(6)?;
                let var_name = format!("{}", self.parse_name(6)?);
                self.add_to_symbol_table(var_type, VariableKind::Jarg, var_name)?;
            }
        }

        self.parse_specific_symbol(')', 4)?;

        // subRoutineBody
        self.parse_subroutine_body()?;

        return Ok(true);
    }

    fn parse_subroutine_body(&mut self) -> Result<(), &'static str> {
        self.var_symbol_table = HashMap::new();
        // {
        self.parse_specific_symbol('{', 6)?;

        // varDec*
        while **self.token_iterator.peek().unwrap() == Token::Keyword(Keyword::Var) {
            self.token_iterator.next();
            // type
            let var_type = self.parse_type(8)?;

            // varName
            let var_name = format!("{}", self.parse_name(8)?);
            self.add_to_symbol_table(var_type.clone(), VariableKind::Jvar, var_name)?;

            // (, varName)*
            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                let var_name = self.parse_name(8)?.to_owned();
                self.add_to_symbol_table(var_type.clone(), VariableKind::Jvar, var_name)?;
            }

            // ;
            self.parse_specific_symbol(';', 8)?;
        }

        self.vm_output += &format!("{nlocals:}\n",nlocals=self.var_symbol_table.len());

        // statements
        while self.parse_statement(8)? {
            // do nothing
        }

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
        self.token_iterator.next();

        self.parse_subroutine_call(xml_indent + 2)?;

        self.parse_specific_symbol(';', xml_indent + 2)?;

        return Ok(());
    }

    fn parse_return_statement(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.vm_output += "pop temp 0\npush constant 0\nreturn";
        self.token_iterator.next();

        if **self.token_iterator.peek().unwrap() != Token::Symbol(';') {
            self.parse_expression(xml_indent + 2)?;
        }
        self.parse_specific_symbol(';', xml_indent + 2)?;

        return Ok(());
    }

    fn parse_expression(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        self.parse_term(xml_indent + 2)?;
        while let Some(operation) = self.get_operation()? {
            self.parse_term(xml_indent + 2)?;
            self.vm_output += &operation.to_vm_command_string();
        }
        return Ok(());
    }

    fn parse_term(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        match self.token_iterator.peek().unwrap() {
            Token::IntConstant(i) => {
                self.vm_output += &format!("push constant {}\n", i);
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
                self.vm_output += "push constant 1\nneg\n";
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::False) => {
                self.vm_output += "push constant 0\n";
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::Null) => {
                self.vm_output += "push constant 0\n";
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

        return Ok(());
    }

    fn parse_expression_list(&mut self, xml_indent: usize) -> Result<usize, &'static str> {
        // (
        self.parse_specific_symbol('(', xml_indent)?;
        let mut num_list_elements = 0;

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') {
            self.parse_expression(xml_indent + 2)?;
            num_list_elements += 1;
        }

        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.parse_specific_symbol(',', xml_indent + 2)?;
            self.parse_expression(xml_indent + 2)?;
            num_list_elements += 1;
        }

        // )
        self.parse_specific_symbol(')', xml_indent)?;
        return Ok(num_list_elements);
    }

    fn parse_subroutine_call(&mut self, xml_indent: usize) -> Result<(), &'static str> {
        let mut fun_name = self.parse_name(xml_indent)?.to_owned();
        // if a dot follows, we have the case className|varName . subRoutineName, otherwise it is just subroutineName
        if **self.token_iterator.peek().unwrap() == Token::Symbol('.') {
            self.parse_specific_symbol('.', xml_indent)?;
            fun_name += &format!(".{}", self.parse_name(xml_indent)?);
        }
        let num_args = self.parse_expression_list(xml_indent)?;
        self.vm_output += &format!("call {} {}\n",fun_name, num_args);

        return Ok(());
    }

    fn get_operation(&mut self) -> Result<Option<JackOperation>, &'static str> {
        let operation = match self.token_iterator.peek().unwrap() {
            Token::Symbol('+') => JackOperation::Add,
            Token::Symbol('-') => JackOperation::Subtract,
            Token::Symbol('*') => JackOperation::Multiply,
            Token::Symbol('/') => JackOperation::Divide,
            Token::Symbol('&') => JackOperation::And,
            Token::Symbol('|') => JackOperation::Or,
            Token::Symbol('<') => JackOperation::Less,
            Token::Symbol('>') => JackOperation::Larger,
            Token::Symbol('=') => JackOperation::Equal,
            _ => return Ok(None),
        };
        self.token_iterator.next();
        Ok(Some(operation))
    }



    fn parse_type(&mut self, xml_indent: usize) -> Result<JackVariableType, &'static str> {
        match self.token_iterator.next().unwrap() {
            Token::Identifier(identifier) => {
                Ok(JackVariableType::Jclass(identifier.to_string()))
            }
            Token::Keyword(Keyword::Int) => {
                Ok(JackVariableType::Jint)
            }
            Token::Keyword(Keyword::Char) => {
                Ok(JackVariableType::Jchar)
            }
            Token::Keyword(Keyword::Boolean) => {
                Ok(JackVariableType::Jboolean)
            }
            _ => Err("Expected a type! Type has to be int, char, boolean, or class name!"),
        }
    }

    fn parse_name(&mut self, xml_indent: usize) -> Result<&str, &'static str> {
        if let Token::Identifier(id) = self.token_iterator.next().unwrap() {
            Ok(id)
        } else {
            Err("Expected a name here!")
        }
    }

    fn parse_specific_symbol(&mut self, c: char, xml_indent: usize) -> Result<(), &'static str> {
        if *self.token_iterator.next().unwrap() == Token::Symbol(c) {
            Ok(())
        } else {
            Err("Expected a different symbol")
        }
    }
}
