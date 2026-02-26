use plumb::cli::run;

fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprint!("{e}");
            1
        }
    };

    std::process::exit(exit_code);
}
