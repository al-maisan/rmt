#[macro_use]
extern crate clap;
use clap::App;
use ini::Ini;
mod config;
use std::process;

fn main() {
   let yaml = load_yaml!("cli.yml");
   let app = App::from_yaml(yaml)
      .about(crate_description!())
      .name(crate_name!())
      .author(crate_authors!())
      .version(crate_version!());
   let matches = app.get_matches();

   if let Some(matches) = matches.subcommand_matches("sample") {
      if let Some(_matches) = matches.subcommand_matches("config") {
         println!("{}", config::gen_config(crate_name!(), crate_version!()));
      }
      if let Some(_matches) = matches.subcommand_matches("template") {
         println!("{}", config::gen_template(crate_name!(), crate_version!()));
      }
   } else if let Some(matches) = matches.subcommand_matches("run") {
      println!("Run mailer tool");
      if matches.is_present("dry_run") {
         println!("* dry run, no action");
      } else {
         println!("* run the mailer");
      }
      let config = matches.value_of("config").unwrap();
      let i = Ini::load_from_file(config).unwrap();
      match config::check(&i) {
         Ok(_) => println!("* config looks good"),
         Err(msg) => {
            println!("!! invalid config -- {:?}", msg);
            process::exit(1)
         }
      }
      let _cfg: config::Config;
      match config::parse(&i) {
         Ok(cfg) => _cfg = cfg,
         Err(msg) => {
            println!("!! config parsing error, {}", msg);
            process::exit(2)
         }
      }
   }
}
