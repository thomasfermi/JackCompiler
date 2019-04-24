#![deny(missing_docs)]
//! JackCompiler
//! Compiles Jack code into virtual machine code. Hack is a computer specified in
//! "The elements of Computing Systems" (a.k.a. "nand2tetris") by Nisan and Schocken.

extern crate clap;
extern crate glob;
use clap::{App, Arg};
use glob::glob;
use std::error::Error;
use std::fs::metadata;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

mod jack_compiler;
mod jack_tokenizer;
pub use jack_compiler::JackCompiler;

fn main() {
    let matches = App::new("JackCompiler")
                          .version("0.1")
                          .author("thomasfermi")
                          .about("Compiles Jack code into virtual machine code. Hack is a computer specified in \"The elements of Computing Systems\" by Nisan and Schocken.")
                          .arg(Arg::with_name("Jack_input")
                               .help("Path to the file containing Jack source code. File extension is jack. ")
                               .required(true)
                               .index(1))
                          .get_matches();

    let input_path_string: String = matches.value_of("Jack_input").unwrap().to_string();

    let mut input_files = vec![];
    // check of user gave directory or single file
    if metadata(&input_path_string).unwrap().is_dir() {
        for entry in glob(&format!("{}/*.jack", input_path_string)).unwrap() {
            match entry {
                Ok(path) => input_files.push(path),

                // if the path matched but was unreadable,
                // thereby preventing its contents from matching
                Err(e) => println!("{:?}", e),
            }
        }
        // check that Sys.vm is part of input_files and also that it is the first element in the list
        input_files.retain(|x| !x.to_str().unwrap().contains("Main.jack")); //TODO: unwrap unsafe
        let sys_vm_path: PathBuf = [&input_path_string, "Main.jack"].iter().collect();
        input_files.push(sys_vm_path);
    } else {
        input_files.push(PathBuf::from(&input_path_string));
    }

    for input_file in input_files {
        let mut jack_source_file_content = String::new();

        {
            let mut file = File::open(&input_file).expect("File not found.");
            file.read_to_string(&mut jack_source_file_content)
                .expect("Could not read file");
        }

        let tokens = jack_tokenizer::tokenize(jack_source_file_content);
        let mut jack_compiler = JackCompiler::new(&tokens);
        let vm_output_string = jack_compiler.compile_class().expect("Parse Error!");

        // Write to output file
        let mut output_file_name = str::replace(
            &input_file.into_os_string().into_string().unwrap(),
            ".jack",
            ".vm",
        );

        let path = Path::new(&output_file_name);
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why.description()),
            Ok(file) => file,
        };

        match file.write_all(vm_output_string.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
            Ok(_) => println!("Successfully wrote xml to {}", display),
        }
    }
}
