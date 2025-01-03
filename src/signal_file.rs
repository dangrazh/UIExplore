use std::env;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::PathBuf;


fn write_to_file(file_name: &PathBuf, text_out: &str) -> Result<(), Error> {
    // let path = "lines.txt";

    let mut output = File::create(file_name)?;
    write!(output, "{}", text_out)?;

    Ok(())
}

fn read_to_string(file_name: &PathBuf) -> Result<String, Error> {
    
    let mut f = File::open(file_name)?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;
    Ok(buffer)
    
}

pub fn create_signal_file() -> Result<(), Error> {
    let file_name = env::temp_dir().join("signal_file.txt");
    let text_out = "terminate";
    write_to_file(&file_name, text_out)?;
    
    Ok(())
}

pub fn termination_signal() -> bool {
    let file_name = env::temp_dir().join("signal_file.txt");
    // let input = read_to_string(&file_name);
    
    if let Ok(text) = read_to_string(&file_name) {
        if text == "terminate" {
            if let Ok(()) = std::fs::remove_file(&file_name) {
                true
            } else {
                false
            }
        } else {
            false
        }

    } else {
        false
    }    
    
}