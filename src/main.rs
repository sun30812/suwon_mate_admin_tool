use std::env;
use suwon_mate_admin_tool::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let program_arguments = ProgramArgument::new(&args).unwrap_or_else(|error| {
        println!("프로그램 오류 발생: {}", error);
        std::process::exit(1);
    });
    if let Err(error) = file_process(program_arguments) {
        println!("응용 프로그램 오류 발생: {}", error);

        std::process::exit(1);
    }
}
