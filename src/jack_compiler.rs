//! jack_compiler
extern crate peek_nth;

use self::peek_nth::{IteratorExt, PeekableNth};

use jack_tokenizer::{Keyword, Token};

use std::collections::HashMap;
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
    pub var_type: JackVariableType,
    pub num: usize,
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

#[derive(Debug, PartialEq, Clone)]
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
    token_iterator: PeekableNth<Iter<'a, Token>>,
    vm_output: String,
    class_name: String,
    field_symbol_table: HashMap<String, SymbolTableEntry>,
    static_symbol_table: HashMap<String, SymbolTableEntry>,
    var_symbol_table: HashMap<String, SymbolTableEntry>,
    arg_symbol_table: HashMap<String, SymbolTableEntry>,
    if_label_num : usize,
    while_label_num : usize,
    currently_in_void_function : bool,
}

impl<'a> JackCompiler<'a> {
    /// Constructor
    pub fn new(tokens: &'a [Token]) -> Self {
        JackCompiler {
            token_iterator: tokens.iter().peekable_nth(),
            vm_output: "".to_string(),
            class_name: "".to_string(),
            field_symbol_table: HashMap::new(),
            static_symbol_table: HashMap::new(),
            var_symbol_table: HashMap::new(),
            arg_symbol_table: HashMap::new(),
            if_label_num : 0,
            while_label_num : 0,
            currently_in_void_function : false,
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

    fn get_vm_code_for_var_name(&self, var_name: &str) -> Result<String,&'static str>  {
        if let Some(entry) = self.var_symbol_table.get(var_name) {
            Ok(format!("local {}", entry.num))
        } else if let Some(entry) = self.arg_symbol_table.get(var_name) {
            Ok(format!("argument {}", entry.num))
        } else if let Some(entry) = self.field_symbol_table.get(var_name) { // ????
            Ok(format!("this {}", entry.num))
        } else if let Some(entry) = self.static_symbol_table.get(var_name) { // ????
            Ok(format!("static {}", entry.num))
        } else {
            return Err("This variable was not defined before");
        }       
    }

    fn get_symbol_table_entry(&self, var_name: &str) -> Result<&SymbolTableEntry,&'static str>  {
        if let Some(entry) = self.var_symbol_table.get(var_name) {
            Ok(entry)
        } else if let Some(entry) = self.arg_symbol_table.get(var_name) {
            Ok(entry)
        } else if let Some(entry) = self.field_symbol_table.get(var_name) { // ????
            Ok(entry)
        } else {
            return Err("This variable is not in the symbol table.");
        }       
    }

    /// Main function. Returns a string containing VM code corresponding to a Jack class
    /// TODO: Write custom Err structs and use them instead of static str
    /// https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html
    pub fn compile_class(&mut self) -> Result<String, &'static str> {
        if Token::Keyword(Keyword::Class) == *self.token_iterator.next().unwrap() {
            // className
            self.class_name = self.parse_name()?.to_owned();
            self.parse_specific_symbol('{')?;

            // classVarDec*
            while self.compile_class_var_dec()? {
                // do nothing
            }

            // subRoutineDec*
            while self.compile_subroutine_dec()? {
                // do nothing
            }

            // }
            self.parse_specific_symbol('}')?;

        } else {
            return Err("This is no class!");
        }

        return Ok(self.vm_output.clone());
    }

    fn compile_class_var_dec(&mut self) -> Result<bool, &'static str> {
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
        let var_type = self.parse_type()?;

        // varName
        let mut var_name = self.parse_name()?.to_owned();

        self.add_to_symbol_table(var_type.clone(), var_kind.clone(), var_name)?;

        // (, varName)*
        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.token_iterator.next(); // peek successful, hence next()

            var_name = self.parse_name()?.to_owned();

            self.add_to_symbol_table(var_type.clone(), var_kind.clone(), var_name)?;
        }

        // ;
        self.parse_specific_symbol(';')?;

        return Ok(true);
    }

    fn compile_subroutine_dec(&mut self) -> Result<bool, &'static str> {
        // forget about last symbol table from last function and initialize new one
        self.arg_symbol_table = HashMap::new();
        self.var_symbol_table = HashMap::new();

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
        self.currently_in_void_function = false;
        match self.token_iterator.next().unwrap() {
            Token::Keyword(Keyword::Int)     => {},
            Token::Keyword(Keyword::Char)    => {},
            Token::Keyword(Keyword::Boolean) => {},
            Token::Keyword(Keyword::Void)    => self.currently_in_void_function = true,
            Token::Identifier(id)            => {
                if function_kind == FunctionKind::Jconstructor && *id != self.class_name {
                    return Err("The return type of a constructor must be the class type!");
                }
            },
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
        self.parse_specific_symbol('(')?;

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') {
            // if function has more than zero arguments
            let var_type = self.parse_type()?;
            let var_name = format!("{}", self.parse_name()?);
            self.add_to_symbol_table(var_type, VariableKind::Jarg, var_name)?;

            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                let var_type = self.parse_type()?;
                let var_name = format!("{}", self.parse_name()?);
                self.add_to_symbol_table(var_type, VariableKind::Jarg, var_name)?;
            }
        }

        self.parse_specific_symbol(')')?;

        // subRoutineBody

        // {
        self.parse_specific_symbol('{')?;

        // varDec*
        while **self.token_iterator.peek().unwrap() == Token::Keyword(Keyword::Var) {
            self.token_iterator.next();
            // type
            let var_type = self.parse_type()?;

            // varName
            let var_name = format!("{}", self.parse_name()?);
            self.add_to_symbol_table(var_type.clone(), VariableKind::Jvar, var_name)?;

            // (, varName)*
            while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
                self.token_iterator.next();
                let var_name = self.parse_name()?.to_owned();
                self.add_to_symbol_table(var_type.clone(), VariableKind::Jvar, var_name)?;
            }

            // ;
            self.parse_specific_symbol(';')?;
        }

        self.vm_output += &format!("{nlocals:}\n",nlocals=self.var_symbol_table.len());


        match function_kind {
            FunctionKind::Jmethod => self.vm_output += "push argument 0\npop pointer 0\n",
            FunctionKind::Jconstructor => {
                self.vm_output += &format!("push constant {}\n", self.field_symbol_table.len());
                self.vm_output += "call Memory.alloc 1\n";
                self.vm_output += "pop pointer 0\n";
            },
            FunctionKind::Jfunction => {},
        }  

        // statements
        while self.compile_statement()? {
            // do nothing
        }

        // }
        self.parse_specific_symbol('}')?;

        return Ok(true);
    }




    fn compile_statement(&mut self) -> Result<bool, &'static str> {
        match self.token_iterator.peek().unwrap() {
            Token::Keyword(kw) => match kw {
                Keyword::Let => self.compile_let_statement()?,
                Keyword::If => self.compile_if_statement()?,
                Keyword::While => self.compile_while_statement()?,
                Keyword::Do => self.compile_do_statement()?,
                Keyword::Return => self.compile_return_statement()?,
                _ => {
                    return Err("Expected a statement beginning with let, if, while, do, or return!")
                }
            },
            _ => return Ok(false),
        }
        return Ok(true);
    }

    fn compile_let_statement(&mut self) -> Result<(), &'static str> {
        self.token_iterator.next();

        // varName
        let var_name = self.parse_name()?.to_owned();        

        // [ expression ]
        let mut left_hand_side_is_array = false;
        if **self.token_iterator.peek().unwrap() == Token::Symbol('[') {
            self.parse_specific_symbol('[')?;
            self.compile_expression()?;
            self.parse_specific_symbol(']')?;
            self.vm_output += &format!("push {}\n", &self.get_vm_code_for_var_name(&var_name)?);
            self.vm_output += "add\n";
            left_hand_side_is_array = true;
        }

        // =
        self.parse_specific_symbol('=')?;
        // expression
        self.compile_expression()?;
        // ;
        self.parse_specific_symbol(';')?;

        if left_hand_side_is_array {
            self.vm_output += "pop temp 0\npop pointer 1\npush temp 0\npop that 0\n";
        } else {
            self.vm_output += &format!("pop {}\n",&self.get_vm_code_for_var_name(&var_name)?);
        }

        return Ok(());
    }

    fn compile_if_statement(&mut self) -> Result<(), &'static str> {
        self.token_iterator.next();

        let current_if_statement_num =  self.if_label_num;
        self.if_label_num += 1;

        // ( expression )
        self.parse_specific_symbol('(')?;

        self.compile_expression()?;

        self.parse_specific_symbol(')')?;

        self.vm_output += &format!("if-goto {}_IF_TRUE{}\n", self.class_name, current_if_statement_num);
        self.vm_output += &format!("goto {}_IF_FALSE{}\n", self.class_name, current_if_statement_num);
        self.vm_output += &format!("label {}_IF_TRUE{}\n", self.class_name, current_if_statement_num);

        // { statements }
        self.parse_specific_symbol('{')?;

        while self.compile_statement()? {
            //do nothing
        }

        self.vm_output += &format!("goto {}_IF_END{}\n", self.class_name, current_if_statement_num);

        self.vm_output += &format!("label {}_IF_FALSE{}\n", self.class_name, current_if_statement_num);

        self.parse_specific_symbol('}')?;

        if Token::Keyword(Keyword::Else) == **self.token_iterator.peek().unwrap() {
            // else
            self.token_iterator.next();
            // { statements }
            self.parse_specific_symbol('{')?;
            while self.compile_statement()? {
                // do nothing
            }
            self.parse_specific_symbol('}')?;
        }
        self.vm_output += &format!("label {}_IF_END{}\n", self.class_name, current_if_statement_num);

        return Ok(());
    }

    fn compile_while_statement(&mut self) -> Result<(), &'static str> {
        self.token_iterator.next();

        let current_while_statement_num =  self.while_label_num;
        self.while_label_num += 1;

        self.vm_output += &format!("label {}_WHILE_EXP{}\n", self.class_name, current_while_statement_num);

        // ( expression )
        self.parse_specific_symbol('(')?;

        self.compile_expression()?;

        self.parse_specific_symbol(')')?;

        self.vm_output += "not\n";
        self.vm_output += &format!("if-goto {}_WHILE_END{}\n", self.class_name, current_while_statement_num);

        // { statements }
        self.parse_specific_symbol('{')?;
        while self.compile_statement()? {
            // do nothing
        }
        self.vm_output += &format!("goto {}_WHILE_EXP{}\n", self.class_name, current_while_statement_num);

        self.parse_specific_symbol('}')?;

        self.vm_output += &format!("label {}_WHILE_END{}\n", self.class_name, current_while_statement_num);
        return Ok(());
    }

    fn compile_do_statement(&mut self) -> Result<(), &'static str> {
        self.token_iterator.next();

        self.compile_subroutine_call()?;

        self.parse_specific_symbol(';')?;

        // in the do statement we do not do anything with the return value from the subroutine call
        // hence pop it somewhere to get rid of it
        self.vm_output += "pop temp 0\n"; 

        return Ok(());
    }

    fn compile_return_statement(&mut self) -> Result<(), &'static str> {
        if self.currently_in_void_function {
            self.vm_output += "push constant 0\n";
        }
        self.token_iterator.next();

        if **self.token_iterator.peek().unwrap() != Token::Symbol(';') {
            self.compile_expression()?;
        }
        self.parse_specific_symbol(';')?;
        self.vm_output += "return\n";

        return Ok(());
    }

    fn compile_expression(&mut self) -> Result<(), &'static str> {
        self.compile_term()?;
        while let Some(operation) = self.get_operation()? {
            self.compile_term()?;
            self.vm_output += &operation.to_vm_command_string();
        }
        return Ok(());
    }

    fn compile_term(&mut self) -> Result<(), &'static str> {
        match self.token_iterator.peek().unwrap() {
            Token::IntConstant(i) => {
                self.vm_output += &format!("push constant {}\n", i);
                self.token_iterator.next();
            }
            Token::StringConstant(s) => {
                self.vm_output += &format!("push constant {}\ncall String.new 1\n", s.len());
                for c in s.chars(){
                    self.vm_output += &format!("push constant {:?}\ncall String.appendChar 2\n", c as u32);
                }
                self.token_iterator.next();
            }
            Token::Keyword(Keyword::True) => {
                self.vm_output += "push constant 0\nnot\n";
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
                self.vm_output += "push pointer 0\n";
                self.token_iterator.next();
            }
            // (expression)
            Token::Symbol('(') => {
                self.parse_specific_symbol('(')?;
                self.compile_expression()?;
                self.parse_specific_symbol(')')?;
            }
            // unaryOp term
            Token::Symbol('-') => {
                self.parse_specific_symbol('-')?;
                self.compile_term()?;
                self.vm_output += "neg\n";
            }
            Token::Symbol('~') => {
                self.parse_specific_symbol('~')?;
                self.compile_term()?;
                self.vm_output += "not\n";
            }
            // varname | varname[expression] | subroutineCall
            Token::Identifier(name) => {
                match **self.token_iterator.peek_nth(1).unwrap() {
                    // varName[expression]
                    Token::Symbol('[') => {
                        self.token_iterator.next();
                        self.parse_specific_symbol('[')?;
                        self.compile_expression()?;
                        self.parse_specific_symbol(']')?;
                        self.vm_output += &format!("push {}\n", &self.get_vm_code_for_var_name(&name)?);
                        self.vm_output += "add\npop pointer 1\npush that 0\n";                      
                    }
                    // subroutinecall, which is var_name.function_name() or function_name()
                    Token::Symbol('.') |  Token::Symbol('(') => {
                        self.compile_subroutine_call()?;
                    }
                    // simply the var_name
                    _ => {
                        self.token_iterator.next();
                        self.vm_output += &format!("push {}\n", &self.get_vm_code_for_var_name(&name)?);
                    }
                }
            }
            Token::Symbol(_s) => return Err("This symbol is not a term"),
            _ => return Err("This token is not a term"),
        }

        return Ok(());
    }

    fn compile_expression_list(&mut self) -> Result<usize, &'static str> {
        // (
        self.parse_specific_symbol('(')?;
        let mut num_list_elements = 0;

        if **self.token_iterator.peek().unwrap() != Token::Symbol(')') {
            self.compile_expression()?;
            num_list_elements += 1;
        }

        while **self.token_iterator.peek().unwrap() == Token::Symbol(',') {
            self.parse_specific_symbol(',')?;
            self.compile_expression()?;
            num_list_elements += 1;
        }

        // )
        self.parse_specific_symbol(')')?;
        return Ok(num_list_elements);
    }

    fn compile_subroutine_call(&mut self) -> Result<(), &'static str> {
        let mut fun_name = self.parse_name()?.to_owned();
        let mut num_args = 0;
        // if a dot follows, we have the case className|varName . subRoutineName, otherwise it is just subroutineName
        if **self.token_iterator.peek().unwrap() == Token::Symbol('.') { // something like Screen.draw()
            self.parse_specific_symbol('.')?;
            // if the left side of the dot has an object from our symbol table, we got to push it to the stack as an additional argument
            match self.get_vm_code_for_var_name(&fun_name) {
                Ok(vm_code) => {
                    if let JackVariableType::Jclass(ref class_name) = self.get_symbol_table_entry(&fun_name)?.var_type{
                        fun_name = class_name.to_owned();
                    }
                    self.vm_output+= &format!("push {}\n", vm_code);
                    num_args = 1;
                },
                Err(_) => { } // fun_name not it symbol table, hence not an object
            }
            fun_name += &format!(".{}", self.parse_name()?);
        } else { // something like draw()
            // Assuming what we call is a method, we need to add the object as argument
            self.vm_output += &format!("push pointer 0\n");
            num_args = 1;
            fun_name = format!("{}.{}",self.class_name, fun_name);
        }
        num_args += self.compile_expression_list()?;
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



    fn parse_type(&mut self) -> Result<JackVariableType, &'static str> {
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

    fn parse_name(&mut self) -> Result<&str, &'static str> {
        if let Token::Identifier(id) = self.token_iterator.next().unwrap() {
            Ok(id)
        } else {
            Err("Expected a name here!")
        }
    }

    fn parse_specific_symbol(&mut self, c: char) -> Result<(), &'static str> {
        if *self.token_iterator.next().unwrap() == Token::Symbol(c) {
            Ok(())
        } else {
            Err("Expected a different symbol")
        }
    }
}
