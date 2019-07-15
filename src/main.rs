#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml)
        .about(crate_description!())
        .name(crate_name!())
        .author(crate_authors!())
        .version(crate_version!());
    let matches = app.get_matches();

    if let Some(matches) = matches.subcommand_matches("sample") {
        println!("Generate sample files");
        if let Some(_matches) = matches.subcommand_matches("config") {
            println!("{}", gen_sample_config(crate_name!(), crate_version!()));
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

fn gen_sample_config(name: &str, version: &str) -> String {
    let sample = format!(
        r#"# {} version {}
#
# anything that follows a hash is a comment
# email address is to the left of the '=' sign, first word after is
# the first name, the rest is the surname
[general]
From="Frodo Baggins" <rts@example.com>
#Cc=weirdo@nsb.gov, cc@example.com
#Reply-To="John Doe" <jd@mail.com>
subject=Hello %FN%!
#attachments=/home/user/atmt1.ics, ../Documents/doc2.txt
[recipients]
# The 'Cc' setting below *redefines* the global 'Cc' value above
jd@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|Cc:-bl@kf.io,info@ex.org
mm@gmail.com=Mickey Mouse|ORG:-Disney   # trailing comment!!
# The 'Cc' setting below *adds* to the global 'Cc' value above
daisy@example.com=Daisy Lila|ORG:-NASA|TITLE:-Dr.|Cc:-+inc@gg.org"#,
        name, version
    );
    sample
}
