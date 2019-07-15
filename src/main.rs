#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("sample") {
        println!("Generate sample files");
        if let Some(_matches) = matches.subcommand_matches("config") {
            println!("!! config sample");
        }
        if let Some(_matches) = matches.subcommand_matches("template") {
            println!("!! template sample");
        }
    } else if let Some(matches) = matches.subcommand_matches("run") {
        println!("Run mailer tool");
        if matches.is_present("dry_run") {
            println!("!! dry run, no action");
        } else {
            println!("!! run the mailer");
        }
    }
}
