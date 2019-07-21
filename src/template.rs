use regex::Regex;
use std::collections::HashSet;

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

/// Constructs a set of `String` from an array of string slices.
fn ss(a: &[&str]) -> HashSet<String> {
   a.iter().map(|w| w.to_string()).collect()
}

#[cfg(test)]
mod tests {
   use super::*;

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
   fn new_with_happy_path() {
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
}
