#[macro_use]
extern crate log;

extern crate clap;

mod cli;
mod error;
mod quic;

fn main() {
    let out = "Hello world!";
    println!("{}", out);

    let args = cli::get_app("1.0").get_matches();

    match args.subcommand() {
        ("client", Some(subcmd)) => {
            println!(
                "Downloading {} -> {} from {}",
                subcmd.value_of("SRC").unwrap(),
                subcmd.value_of("DEST").unwrap(),
                subcmd.value_of("server").unwrap()
            );
        }

        ("server", Some(subcmd)) => {
            println!(
                "Serving {} on {}",
                subcmd.value_of("PATH").unwrap(),
                subcmd.value_of("SERVER").unwrap()
            );
        }

        _ => {
            cli::get_app("1.0").print_help().unwrap();
        }
    }
}
