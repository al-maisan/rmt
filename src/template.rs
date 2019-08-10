use crate::config::Recipient;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io;

#[derive(Debug)]
/// The `Template` struct holds the template data.
pub struct Template {
   /// This is the recipient's email address
   text: String,
   /// This is a map with miscellaneous optional metadata that was defined for the recipient in
   /// question
   keys: HashSet<String>,
}

impl PartialEq for Template {
   /// Makes it possible to compare instances of `Template`
   fn eq(&self, other: &Self) -> bool {
      self.text == other.text && self.keys == other.keys
   }
}

pub fn instantiate(template_path: &str) -> Result<Template, io::Error> {
   let contents = fs::read_to_string(template_path)?;
   return Ok(new(&contents));
}

pub fn new(template: &str) -> Template {
   let mut result = Template {
      text: template.to_string(),
      keys: HashSet::new(),
   };
   let re = Regex::new(r"%(\w+)%").expect("internal error, invalid regex");
   for cap in re.captures_iter(template) {
      result.keys.insert(cap[1].to_string());
   }
   result
}

impl Template {
   pub fn check_recipents(&self, recipients: &Vec<Recipient>) -> Result<(), Vec<String>> {
      let auto_keys: HashSet<String> = ["_EA", "_FN", "_LN", "_TN", "_TV"]
         .iter()
         .map(|w| w.to_string())
         .collect();
      let user_defined_keys: HashSet<String> = self
         .keys
         .iter()
         .cloned()
         .filter(|k| !auto_keys.contains(k))
         .collect();
      let mut errors = vec![];
      for rcp in recipients {
         let rcp_keys: HashSet<String> = rcp.data.keys().cloned().collect();
         if !user_defined_keys.is_subset(&rcp_keys) {
            let mut missing_keys: Vec<String> = user_defined_keys
               .iter()
               .cloned()
               .filter(|k| !rcp_keys.contains(k))
               .collect();
            missing_keys.sort();
            errors.push(format!(
               "{} is missing the following key(s): {}",
               rcp.email,
               missing_keys.as_slice().join(", ")
            ));
         }
      }
      if errors.len() > 0 {
         return Err(errors);
      } else {
         return Ok(());
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use crate::config::sa;
   use crate::config::sm;

   /// Constructs a set of `String` from an array of string slices.
   fn ss(a: &[&str]) -> HashSet<String> {
      a.iter().map(|w| w.to_string()).collect()
   }

   #[test]
   fn new_with_empty_string() {
      let expected = Template {
         text: String::from(""),
         keys: HashSet::new(),
      };
      assert_eq!(expected, new(""));
   }

   #[test]
   fn new_with_no_keys() {
      let template = "Hello Sir! May I get you interested in..?";
      let expected = Template {
         text: String::from(template),
         keys: HashSet::new(),
      };
      assert_eq!(expected, new(template));
   }

   #[test]
   fn new_happy_case() {
      let template = r#"FN / LN / EA = first name / last name / email address

Hello %FN% // %LN%, how are things going at %ORG%?
this is your email: %EA% :)
have a nice day %FN% %LN%!!


Sent with rmt version 0.1.2, see https://301.mx/rmt for details"#;
      let expected = Template {
         text: String::from(template),
         keys: ss(&["EA", "FN", "LN", "ORG"]),
      };
      assert_eq!(expected, new(template));
   }

   #[test]
   fn new_with_invalid_keys() {
      let template = "Hello Sir %FN%! How about %FN or EA% / %%HM%??";
      let expected = Template {
         text: String::from(template),
         keys: ss(&["FN", "HM"]),
      };
      assert_eq!(expected, new(template));
   }

   #[test]
   fn new_with_empty_keys() {
      let template = "Hello Sir %FN%! How about %FN / %% / % / %HM%%??";
      let expected = Template {
         text: String::from(template),
         keys: ss(&["FN", "HM"]),
      };
      assert_eq!(expected, new(template));
   }

   #[test]
   fn new_with_keys_containing_digits() {
      let template = "Hello Sir %FN%! How about %FN or EA% / %%H3%??";
      let expected = Template {
         text: String::from(template),
         keys: ss(&["FN", "H3"]),
      };
      assert_eq!(expected, new(template));
   }

   #[test]
   fn new_with_keys_containing_non_alphanumerics() {
      let template = "Hello Sir %FN%! How about %--% or %%% / %H3%%??";
      let expected = Template {
         text: String::from(template),
         keys: ss(&["FN", "H3"]),
      };
      assert_eq!(expected, new(template));
   }

   #[test]
   fn check_recipents_with_1_missing_key() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("ORG", "NASA"), ("TITLE", "Dr."), ("cc", "+inc@gg.org")]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[
            ("ORG", "EFF"),
            ("TITLE", "PhD"),
            ("cc", "bl@kf.io,info@ex.org"),
         ]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("ORG", "Disney")]),
      });
      let template = new("Missing key: %MK%");
      let expected: Vec<String> = sa(&[
         "daisy@example.com is missing the following key(s): MK",
         "jd@example.com is missing the following key(s): MK",
         "mm@gmail.com is missing the following key(s): MK",
      ]);
      assert_eq!(Err(expected), template.check_recipents(&recipients));
   }

   #[test]
   fn check_recipents_with_multiple_missing_key() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("MK", "NASA"), ("TITLE", "Dr."), ("cc", "+inc@gg.org")]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[
            ("ORG", "EFF"),
            ("TITLE", "PhD"),
            ("M2", "bl@kf.io,info@ex.org"),
         ]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("ORG", "Disney")]),
      });
      let template = new("Missing key: %MK% %M2% %m3%");
      let expected: Vec<String> = sa(&[
         "daisy@example.com is missing the following key(s): M2, m3",
         "jd@example.com is missing the following key(s): MK, m3",
         "mm@gmail.com is missing the following key(s): M2, MK, m3",
      ]);
      assert_eq!(Err(expected), template.check_recipents(&recipients));
   }

   #[test]
   fn check_recipents_happy_case() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("ORG", "NASA"), ("TITLE", "Dr."), ("cc", "+inc@gg.org")]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[("ORG", "EFF"), ("TITLE", "PhD")]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("ORG", "Disney")]),
      });
      let template = new("only key: %ORG%");
      assert_eq!(Ok(()), template.check_recipents(&recipients));
   }

   #[test]
   fn check_recipents_happy_case_no_keys_in_template() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("ORG", "NASA"), ("TITLE", "Dr."), ("cc", "+inc@gg.org")]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[("ORG", "EFF"), ("TITLE", "PhD")]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("ORG", "Disney")]),
      });
      let template = new("no keys in template");
      assert_eq!(Ok(()), template.check_recipents(&recipients));
   }

   #[test]
   fn check_recipents_happy_case_with_auto_keys_only() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[]),
      });
      let template = new("auto keys only: %_FN%, %_LN%, %_EA% !!");
      assert_eq!(Ok(()), template.check_recipents(&recipients));
   }

   #[test]
   fn check_recipents_happy_case_with_mixed_keys() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("_USER_DEFINED", "dec")]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[("_USER_DEFINED", "jec")]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("_USER_DEFINED", "mgc")]),
      });
      let template = new("auto keys only: %_FN%, %_USER_DEFINED%!!");
      assert_eq!(Ok(()), template.check_recipents(&recipients));
   }

   #[test]
   fn check_recipents_failure_with_mixed_keys() {
      let mut recipients = Vec::new();
      recipients.push(Recipient {
         email: String::from("daisy@example.com"),
         names: sa(&["Daisy", "Lila"]),
         data: sm(&[("USER_DEFINED", "dec")]),
      });
      recipients.push(Recipient {
         email: String::from("jd@example.com"),
         names: sa(&["John", "Doe", "Jr."]),
         data: sm(&[("_USER_DEFINED", "jec")]),
      });
      recipients.push(Recipient {
         email: String::from("mm@gmail.com"),
         names: sa(&["Mickey", "Mouse"]),
         data: sm(&[("_USER_DEFINED", "mgc")]),
      });
      let template = new("auto keys only: %_FN%, %_USER_DEFINED%!!");
      let expected: Vec<String> =
         sa(&["daisy@example.com is missing the following key(s): _USER_DEFINED"]);
      assert_eq!(Err(expected), template.check_recipents(&recipients));
   }
}
