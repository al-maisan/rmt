pub fn check(cfg: &ini::Ini) -> Result<usize, String> {
   let sections = ["general", "recipients"];
   let mut num_recipients = 0;

   for s in sections.iter() {
      match cfg.section(Some(s.to_string())) {
         Some(props) => {
            if *s == "general" {
               if !props.contains_key("From") {
                  return Err(String::from("No from header in the general section"));
               }
            }
            if *s == "recipients" {
               num_recipients = props.len();
               if num_recipients == 0 {
                  return Err(String::from("No email recipients found in config file"));
               }
            }
         }
         None => {
            return Err(format!("No *{}* section in config file", s));
         }
      }
   }
   Ok(num_recipients)
}

pub fn gen_config(name: &str, version: &str) -> String {
   format!(
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
   )
}

pub fn gen_template(name: &str, version: &str) -> String {
   format!(
      r#"FN / LN / EA = first name / last name / email address

Hello %FN% // %LN%, how are things going at %ORG%?
this is your email: %EA% :)


Sent with {} version {}, see https://301.mx/{} for details"#,
      name, version, name
   )
}

#[cfg(test)]
mod tests {
   use super::*;
   use ini::Ini;
   use std::io::{Error, Write};
   use tempfile::NamedTempFile;

   fn prep_config(content: &str) -> Result<ini::Ini, Error> {
      let mut tf = NamedTempFile::new()?;

      // Write some test data to the first handle.
      tf.write_all(content.as_bytes())?;
      let cfg = Ini::load_from_file(tf.path()).unwrap();
      Ok(cfg)
   }
   #[test]
   fn check_with_empty_file() {
      let cfg = prep_config("").expect("Failed to set up config");;
      assert_eq!(
         Err(String::from("No *general* section in config file")),
         check(&cfg)
      );
   }
   #[test]
   fn check_with_no_recipients_section() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(
         Err(String::from("No *recipients* section in config file")),
         check(&cfg)
      );
   }
   #[test]
   fn check_with_no_recipients() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
[recipients]"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(
         Err(String::from("No email recipients found in config file")),
         check(&cfg)
      );
   }
   #[test]
   fn check_with_no_from_in_empty_general() {
      let file = r#"
[general]
# this is a comment
[recipients]"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(
         Err(String::from("No from header in the general section")),
         check(&cfg)
      );
   }
   #[test]
   fn check_with_no_from_in_non_empty_general() {
      let file = r#"
[general]
P1=a
P2=b
# this is a comment
[recipients]"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(
         Err(String::from("No from header in the general section")),
         check(&cfg)
      );
   }
   #[test]
   fn check_with_happy_path() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
[recipients]
a@b.com=A B
c@d.com=C D"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(Ok(2), check(&cfg));
   }
}
