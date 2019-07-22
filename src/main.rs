#[macro_use]
extern crate clap;
use clap::App;
mod config;
mod template;

macro_rules! ee {
   ($res:expr) => {
      match $res {
         Ok(v) => v,
         Err(m) => {
            println!("!! error: {}", m);
            ::std::process::exit(1)
         }
      }
   };
}

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
      let config_path = matches.value_of("config").unwrap();
      let template_path = matches.value_of("template").unwrap();

      let cfg = ee!(config::instantiate(
         config_path,
         crate_name!(),
         crate_version!()
      ));
      let tmpl = ee!(template::instantiate(template_path));

      match tmpl.check_recipents(&cfg.recipients) {
         Ok(()) => println!("* recpient data looks good"),
         Err(errors) => {
            println!("!! error some recipient(s) are missing data needed in the template");
            for err in errors {
               println!("    - {}", err)
            }
            ::std::process::exit(2)
         }
      }
   }
}
