/// The `config` module implements the logic for parsing config files.
use ini::Ini;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
/// The `Config` struct holds the contents of the config file after the latter was parsed
/// successfully.
pub struct Config {
   /// The 'From' email header value (a valid email address)
   from: String,
   /// The email subject
   subject: String,
   /// A list of 'Cc' email addresses
   cc: Vec<String>,
   /// A list of 'Reply-To' email addresses
   replyto: Vec<String>,
   /// The version of the tool
   version: String,
   /// A list of recipients who should recaive the email
   recipients: Vec<Recipient>,
}

impl PartialEq for Config {
   /// Makes it possible to compare instances of `Config`
   fn eq(&self, other: &Self) -> bool {
      self.from == other.from
         && self.replyto == other.replyto
         && self.cc == other.cc
         && self.subject == other.subject
         && self.recipients == other.recipients
   }
}

impl ToString for Config {
   /// Makes it possible to print instances of `Config`
   fn to_string(&self) -> String {
      let mut result = format!("from: {}, subject: {}", self.from, self.subject);
      if self.cc.len() > 0 {
         result.push_str(format!(", cc: {}", self.cc.join(", ")).as_ref());
      }
      if self.replyto.len() > 0 {
         result.push_str(format!(", replyto: {}", self.replyto.join(", ")).as_ref());
      }
      result.push_str(format!(", recipients: {{{}}}", self.recipients[0].to_string()).as_ref());
      for recipient in self.recipients.iter().skip(1) {
         result.push_str(format!(", {{{}}}", recipient.to_string()).as_ref());
      }
      result
   }
}

#[derive(Debug)]
/// The `Recipient` struct holds per-recipient data
pub struct Recipient {
   /// This is the recipient's email address
   pub email: String,
   /// This is a list of the recipient's names
   pub names: Vec<String>,
   /// This is a map with miscellaneous optional metadata that was defined for the recipient in
   /// question
   pub data: HashMap<String, String>,
}

impl PartialEq for Recipient {
   /// Makes it possible to compare instances of `Recipient`
   fn eq(&self, other: &Self) -> bool {
      self.email == other.email && self.names == other.names && self.data == other.data
   }
}

impl ToString for Recipient {
   /// Makes it possible to print instances of `Recipient`
   fn to_string(&self) -> String {
      let mut dv = Vec::new();
      for (key, val) in self.data.iter() {
         dv.push(format!("{} => {}", key, val));
      }
      dv.sort();
      format!(
         "email: {}, names: {}, data: {}",
         self.email,
         self.names.join(", "),
         dv.join(", "),
      )
   }
}

pub fn instantiate(config_path: &str) -> Result<Config, String> {
   let i = Ini::load_from_file(config_path).unwrap();
   check(&i)?;
   parse(&i)
}

/// Constructs a list of `String` from an array of string slices.
pub fn sa(a: &[&str]) -> Vec<String> {
   a.iter().map(|w| w.to_string()).collect()
}

/// Constructs a map of `String` from an array 2-tuples with string slices.
pub fn sm(a: &[(&str, &str)]) -> HashMap<String, String> {
   let mut result: HashMap<String, String> = HashMap::new();
   for (k, v) in a.iter() {
      result.insert(k.to_string(), v.to_string());
   }
   result
}

/// Top-level configuration parsing function.
pub fn parse(cfg: &ini::Ini) -> Result<Config, String> {
   let mut result = parse_general(cfg)?;
   result.recipients = parse_recipients(cfg)?;
   Ok(result)
}

/// Takes a string with comma-delimited email addresses and checks their validity.
///
/// If they are all valid returns them as a list of strings. Returns various error messages in the
/// opposite case. See the unit tests for details.
fn check_emails(header: &str, emails: &str) -> Result<Vec<String>, String> {
   let mut valid = Vec::new();
   let mut invalid = Vec::new();
   let data: Vec<String> = emails
      .split(",")
      .map(|w| w.trim())
      .filter(|w| w.len() > 0)
      .map(|w| w.to_string())
      .collect();
   if data.len() == 0 {
      return Err(format!("no emails for *{}* header", header));
   }
   for email in data {
      if check_email(&email) {
         valid.push(email)
      } else {
         invalid.push(email)
      }
   }
   if invalid.len() > 0 {
      invalid.sort();
      return Err(format!(
         "invalid *{}* email(s): {}",
         header,
         invalid.join(", ")
      ));
   }
   valid.sort();
   Ok(valid)
}

/// Parses the `[general]` config file section, returns a `Config` object that has everything but
/// the recipient data if successfull.
fn parse_general(cfg: &ini::Ini) -> Result<Config, String> {
   let mut result = Config {
      from: String::from(""),
      replyto: vec![],
      cc: vec![],
      subject: String::from(""),
      version: String::from(""),
      recipients: vec![],
   };
   let section = cfg.section(Some(String::from("general"))).unwrap();

   let keys: Vec<&String> = section.keys().collect();

   for key in keys {
      let val = section.get(key).unwrap();
      match key.as_ref() {
         "From" | "from" => {
            if !check_email(val) {
               return Err(format!("invalid *From* email: {}", val));
            } else {
               result.from = val.to_string();
            }
         }
         "Reply-To" | "Reply-to" => result.replyto = check_emails(key, val)?,
         "cc" | "Cc" | "CC" => result.cc = check_emails(key, val)?,
         "Subject" | "subject" => result.subject = val.to_string(),
         _ => return Err(format!("invalid configuration datum: *{}*", key)),
      }
   }
   Ok(result)
}

/// Implements a crude, basic sanity check for email addresses. Yay, regular expressions :-P
fn check_email(email: &str) -> bool {
   let re_long = Regex::new(r#"^("\s*)?(\S+\s+)*(\S+)\s*"?\s+<\S+@\S+\.\S+>$"#).unwrap();
   let re = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
   re_long.is_match(email.to_string().trim()) || re.is_match(email.to_string().trim())
}

/// Parses the optional per-recipient data (delimited by `':-'`) if present.
fn parse_recipient_data(rdata: &Vec<&str>) -> Result<HashMap<String, String>, String> {
   let mut result: Vec<(&str, &str)> = Vec::new();
   for rd in rdata.iter() {
      // split the data, example: "cc:-+inc@gg.org"
      let data = rd.split(":-").map(|w| w.trim()).collect::<Vec<&str>>();
      match data.as_slice() {
         [""] => continue,
         [key, val] => {
            if key.len() == 0 && val.len() == 0 {
               continue;
            }
            if key.len() == 0 {
               return Err(format!("no key for datum ({})", val));
            }
            if val.len() == 0 {
               return Err(format!("empty value for key ({})", key));
            }
            result.push((key, val));
         }
         _ => return Err(format!("invalid recipient data ({})", rd)),
      }
   }
   Ok(sm(&result))
}

/// Parses the `[recipients]` config file section.
fn parse_recipients(cfg: &ini::Ini) -> Result<Vec<Recipient>, String> {
   let mut result: Vec<Recipient> = Vec::new();
   let section = cfg.section(Some(String::from("recipients"))).unwrap();

   // we want a stable sort order of the recipient data
   let mut keys: Vec<&String> = section.keys().collect();
   keys.sort();

   for key in keys {
      let val = section.get(key).unwrap();
      if !check_email(key) {
         return Err(format!("invalid email: {}", key));
      }
      // split recipient data, example:
      // John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org
      let mut data: Vec<&str> = val
         .split("|")
         .map(|w| w.trim())
         .filter(|w| w.len() > 0)
         .collect();
      if data.len() == 0 {
         return Err(format!("invalid data for email: {}", key));
      }
      // split the first entry in the recipient data i.e. the names
      let names: Vec<String> = data
         .remove(0)
         .split_ascii_whitespace()
         .filter(|w| w.len() > 0)
         .map(|n| n.to_string())
         .collect();
      // parse the remainder of the recipient data
      match parse_recipient_data(&data) {
         Ok(rd) => result.push(Recipient {
            email: key.to_string(),
            names: names,
            data: rd,
         }),
         Err(msg) => return Err(format!("invalid recipient data for {} ({})", key, msg)),
      }
   }
   return Ok(result);
}

/// Very basic sanity checks on the config.
///
/// Does it have the general/recipients sections and does the former have a `From` and a `Subject`?
pub fn check(cfg: &ini::Ini) -> Result<usize, String> {
   let sections = sa(&["general", "recipients"]);
   let mut num_recipients = 0;

   for s in sections {
      match cfg.section(Some(s.to_string())) {
         Some(props) => {
            if s == "general" {
               if !props.contains_key("From") && !props.contains_key("from") {
                  return Err(String::from("No *From* header in the general section"));
               }
               if !props.contains_key("Subject") && !props.contains_key("subject") {
                  return Err(String::from("No *Subject* in the general section"));
               }
            }
            if s == "recipients" {
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

/// Generates a configuration for a mailing campaign for a user to tweak as needed.
pub fn gen_config(name: &str, version: &str) -> String {
   format!(
      r#"# {} version {}
#
# anything that follows a hash is a comment
# email address is to the left of the '=' sign, first word after is
# the first name, the rest is the surname
[general]
From="Frodo Baggins" <rts@example.com>
#cc=weirdo@nsb.gov, cc@example.com
#Reply-To="John Doe" <jd@mail.com>
subject=Hello %FN%!
#attachments=/home/user/atmt1.ics, ../Documents/doc2.txt
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
jd@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org
mm@gmail.com=Mickey Mouse|ORG:-Disney   # trailing comment!!
# The 'cc' setting below *adds* to the global 'cc' value above
daisy@example.com=Daisy Lila|ORG:-NASA|TITLE:-Dr.|cc:-+inc@gg.org"#,
      name, version
   )
}

/// Generates a template for a mailing campaign for a user to tweak as needed.
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
Subject=hello world!
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
Subject=hello world!
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
         Err(String::from("No *From* header in the general section")),
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
         Err(String::from("No *From* header in the general section")),
         check(&cfg)
      );
   }

   #[test]
   fn check_with_no_subject() {
      let file = r#"
[general]
From=a
# this is a comment
[recipients]"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(
         Err(String::from("No *Subject* in the general section")),
         check(&cfg)
      );
   }

   #[test]
   fn check_with_happy_path() {
      let file = r#"
[general]
From=abc@def.com
Subject=hello world!
# this is a comment
[recipients]
a@b.com=A B
c@d.com=C D"#;
      let cfg = prep_config(file).expect("Failed to set up config");;
      assert_eq!(Ok(2), check(&cfg));
   }

   #[test]
   fn parse_recipients_with_happy_path() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
jd@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org
mm@gmail.com=Mickey Mouse|ORG:-Disney   # trailing comment!!
# The 'cc' setting below *adds* to the global 'cc' value above
daisy@example.com=Daisy Lila|ORG:-NASA|TITLE:-Dr.|cc:-+inc@gg.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let mut expected = Vec::new();
      expected.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("ORG", "NASA"), ("TITLE", "Dr."), ("cc", "+inc@gg.org")]),
      });
      expected.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[
            ("ORG", "EFF"),
            ("TITLE", "PhD"),
            ("cc", "bl@kf.io,info@ex.org"),
         ]),
      });
      expected.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("ORG", "Disney")]),
      });
      assert_eq!(
         expected,
         parse_recipients(&cfg).expect("This should not fail")
      );
   }

   #[test]
   fn recipients_to_string() {
      let r = Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[
            ("ORG", "EFF"),
            ("TITLE", "PhD"),
            ("cc", "bl@kf.io,info@ex.org"),
         ]),
      };
      assert_eq!("email: jd@example.com, names: John, Doe, Jr., data: ORG => EFF, TITLE => PhD, cc => bl@kf.io,info@ex.org", r.to_string());
   }

   #[test]
   fn check_email_happy_case() {
      assert_eq!(true, check_email("abx@yajo.co.uk"));
   }

   #[test]
   fn check_email_with_plus_char_happy_case() {
      assert_eq!(true, check_email("abx+alias@yajo.co.uk"));
   }

   #[test]
   fn check_email_with_leading_trailing_whitespace_happy_case() {
      assert_eq!(true, check_email("      abx+alias@yajo.co.uk      "));
   }

   #[test]
   fn check_email_with_failure() {
      assert_eq!(false, check_email("@yajo.co.uk"));
   }

   #[test]
   fn check_email_with_whitespace_failure() {
      assert_eq!(false, check_email("    @yajo.co.uk"));
      assert_eq!(false, check_email("hello@   .uk  "));
      assert_eq!(false, check_email("hello@"));
      assert_eq!(false, check_email("@"));
      assert_eq!(false, check_email("hello@      "));
   }

   #[test]
   fn check_email_with_long_form_and_quotes() {
      assert_eq!(true, check_email(r#""Frodo Baggins" <rts@example.com>"#));
   }

   #[test]
   fn check_email_with_long_form_and_no_quotes() {
      assert_eq!(true, check_email(r#"Frodo Baggins <rts@example.com>"#));
   }

   #[test]
   fn parse_recipients_with_invalid_email() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("invalid email: @example.com"));
      assert_eq!(expected, parse_recipients(&cfg));
   }

   #[test]
   fn parse_recipients_with_invalid_data() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
a@example.com="#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("invalid data for email: a@example.com"));
      assert_eq!(expected, parse_recipients(&cfg));
   }

   #[test]
   fn parse_recipients_with_invalid_non_name_data_no_value() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
a@example.com=A B C|ORG:-"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from(
         "invalid recipient data for a@example.com (empty value for key (ORG))",
      ));
      assert_eq!(expected, parse_recipients(&cfg));
   }

   #[test]
   fn parse_recipients_with_invalid_non_name_data_no_key() {
      let file = r#"
[general]
From=abc@def.com
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
a@example.com=A B C|:-Disney"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from(
         "invalid recipient data for a@example.com (no key for datum (Disney))",
      ));
      assert_eq!(expected, parse_recipients(&cfg));
   }

   #[test]
   fn parse_recipient_data_happy_case() {
      let expected = sm(&[
         ("ORG", "EFF"),
         ("TITLE", "PhD"),
         ("cc", "bl@kf.io,info@ex.org"),
      ]);
      let mut args: Vec<&str> =
         "jd@example.com=John Doe Jr.|ORG:-EFF|   TITLE:-PhD|cc:-       bl@kf.io,info@ex.org"
            .split("|")
            .collect();

      args.remove(0);
      assert_eq!(Ok(expected), parse_recipient_data(&args));
   }

   #[test]
   fn parse_recipient_data_happy_case_with_empty_key_and_value() {
      let expected = sm(&[("ORG", "EFF"), ("cc", "bl@kf.io,info@ex.org")]);
      let mut args: Vec<&str> =
         "jd@example.com=John Doe Jr.|ORG:-EFF|:-|cc:-       bl@kf.io,info@ex.org"
            .split("|")
            .collect();

      args.remove(0);
      assert_eq!(Ok(expected), parse_recipient_data(&args));
   }

   #[test]
   fn parse_recipient_data_failure_case_with_empty_key() {
      let mut args: Vec<&str> =
         "jd@example.com=John Doe Jr.|:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"
            .split("|")
            .collect();

      args.remove(0);
      assert_eq!(
         Err(String::from("no key for datum (EFF)")),
         parse_recipient_data(&args)
      );
   }

   #[test]
   fn parse_recipient_data_failure_case_with_empty_value() {
      let mut args: Vec<&str> =
         "jd@example.com=John Doe Jr.|ORG:-   |TITLE:-PhD|cc:-bl@kf.io,info@ex.org"
            .split("|")
            .collect();

      args.remove(0);
      assert_eq!(
         Err(String::from("empty value for key (ORG)")),
         parse_recipient_data(&args)
      );
   }

   #[test]
   fn parse_recipient_data_happy_case_with_empty_rdata() {
      let expected = sm(&[("TITLE", "PhD")]);
      let mut args: Vec<&str> = "jd@example.com=John Doe Jr.||TITLE:-PhD|"
         .split("|")
         .collect();

      args.remove(0);
      assert_eq!(Ok(expected), parse_recipient_data(&args));
   }

   #[test]
   fn parse_recipient_data_failure_case_with_invalid_format() {
      let mut args: Vec<&str> = "jd@example.com=John Doe Jr.|R:-E:-K:-T|cc:-bl@kf.io,info@ex.org"
         .split("|")
         .collect();

      args.remove(0);
      assert_eq!(
         Err(String::from("invalid recipient data (R:-E:-K:-T)")),
         parse_recipient_data(&args)
      );
   }

   #[test]
   fn parse_general_with_invalid_from_email() {
      let file = r#"
[general]
From=abc@defcom
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("invalid *From* email: abc@defcom"));
      assert_eq!(expected, parse_general(&cfg));
   }

   #[test]
   fn parse_general_with_invalid_reply_to_email() {
      let file = r#"
[general]
From=abc@def.com
Reply-To=no@one
# this is a comment
cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("invalid *Reply-To* email(s): no@one"));
      assert_eq!(expected, parse_general(&cfg));
   }

   #[test]
   fn parse_general_with_invalid_cc_email() {
      let file = r#"
[general]
From=abc@def.com
Reply-To=no@one.org
# this is a comment
cc=dd@examplecom, weirdo@nsb.gov, oh!no!
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("invalid *cc* email(s): dd@examplecom, oh!no!"));
      assert_eq!(expected, parse_general(&cfg));
   }

   #[test]
   fn parse_general_with_empty_cc_email() {
      let file = r#"
[general]
From=abc@def.com
Reply-To=no@one.org
# this is a comment
Cc=
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("no emails for *Cc* header"));
      assert_eq!(expected, parse_general(&cfg));
   }

   #[test]
   fn parse_general_with_invalid_config_datum() {
      let file = r#"
[general]
From=abc@def.com
Reply-To=no@one.org
# this is a comment
blah=invalid
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = Err(String::from("invalid configuration datum: *blah*"));
      assert_eq!(expected, parse_general(&cfg));
   }

   #[test]
   fn parse_happy_case() {
      let file = r#"
#
# anything that follows a hash is a comment
# email address is to the left of the '=' sign, first word after is
# the first name, the rest is the surname
[general]
From="Frodo Baggins" <rts@example.com>
cc=weirdo@nsb.gov, cc@example.com
Reply-To="John Doe" <jd@mail.com>
subject=Hello %FN%!
#attachments=/home/user/atmt1.ics, ../Documents/doc2.txt
[recipients]
# The 'cc' setting below *redefines* the global 'cc' value above
jd@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|cc:-bl@kf.io,info@ex.org
mm@gmail.com=Mickey Mouse|ORG:-Disney   # trailing comment!!
# The 'cc' setting below *adds* to the global 'cc' value above
daisy@example.com=Daisy Lila|ORG:-NASA|TITLE:-Dr.|cc:-+inc@gg.org"#;
      let cfg = prep_config(file).expect("Failed to set up config");
      let expected = "from: Frodo Baggins <rts@example.com>, subject: Hello %FN%!, cc: cc@example.com, weirdo@nsb.gov, replyto: John Doe <jd@mail.com>, recipients: {email: daisy@example.com, names: Daisy, Lila, data: ORG => NASA, TITLE => Dr., cc => +inc@gg.org}, {email: jd@example.com, names: John, Doe, Jr., data: ORG => EFF, TITLE => PhD, cc => bl@kf.io,info@ex.org}, {email: mm@gmail.com, names: Mickey, Mouse, data: ORG => Disney}";
      let actual = parse(&cfg).expect("Failed to parse config");

      assert_eq!(expected, actual.to_string());
   }
}
