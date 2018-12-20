#![deny(missing_docs)]
//! HackVirtualMachineTranslator
//! Converts Hack Virtual Machine code to Hack Assembly code. Hack is a computer specified in
//! "The elements of Computing Systems" (a.k.a. "nand2tetris") by Nisan and Schocken.


extern crate clap;
extern crate glob;
use clap::{Arg, App};
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::fs::metadata;
use glob::glob;




mod jack_tokenizer;
pub use jack_tokenizer::JackTokenizer;

fn main() {
    let matches = App::new("JackTokenizer")
                          .version("0.1")
                          .author("thomasfermi")
                          .about("Tokenizes Jack code. Hack is a computer specified in \"The elements of Computing Systems\" by Nisan and Schocken.")
                          .arg(Arg::with_name("Jack_input")
                               .help("Path to the file containing Jack source code. File extension is jack. ")
                               .required(true)
                               .index(1))
                          .get_matches();

    let input_path_string : String = matches.value_of("Jack_input").unwrap().to_string();

    let mut content = String::new();

    {
        let mut file = File::open(&input_path_string).expect("File not found.");
        file.read_to_string(&mut content).expect("Could not read file");
    }


    //let mut vm_translator = VirtualMachineTranslator::new(&file_content_list);
    let mut jack_tokenizer = JackTokenizer::new(content);
    let xml_file = jack_tokenizer.tokenize();

    // Write to output file
    let mut output_file_name = str::replace(&input_path_string,".jack", "_self.xml");



    {
        let path = Path::new(&output_file_name);
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}",
                            display,
                            why.description()),
            Ok(file) => file,
        };

        match file.write_all(xml_file.as_bytes()) {
            Err(why) => {
                panic!("couldn't write to {}: {}", display,
                                                why.description())
            },
            Ok(_) => println!("Successfully wrote machine code to {}", display),
        }
    }

}
