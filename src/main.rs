extern crate clap;
use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("Rust mail tool")
        .version("0.1.0")
        .author("Muharem Hrnjadovic <muharem@linux.com>")
        .about("Facilitates the sending of mass emails using templates")
        .subcommand(SubCommand::with_name("sample").about("Generates sample config or template"))
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs mailer tool")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("config file for an email campaign")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("template")
                        .short("t")
                        .long("template")
                        .value_name("FILE")
                        .help("template file for an email campaign")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dry_run")
                        .short("n")
                        .long("dry-run")
                        .value_name("FILE")
                        .help("just show what would be done, no action")
                        .takes_value(false),
                ),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("not set");
    println!("Value for config: {}", config);
    let template = matches.value_of("template").unwrap_or("not set");
    println!("Value for template: {}", template);

    if let Some(_matches) = matches.subcommand_matches("sample") {
        println!("Generate sample files");
    } else if let Some(matches) = matches.subcommand_matches("run") {
        println!("Run mailer tool");
        if matches.is_present("dry_run") {
            println!("!! dry run, no action");
        } else {
            println!("!! run the mailer");
        }
    }
}
