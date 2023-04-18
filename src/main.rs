use clap::Parser;

use suwon_mate_admin_tool::*;

fn main() {
    let program_arguments = ProgramArgument::parse();
    if let Err(error) = file_process(program_arguments) {
        println!("응용 프로그램 오류 발생: {}", error);

        std::process::exit(1);
    }
}
