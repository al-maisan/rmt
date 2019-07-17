use regex::Regex;
use std::collections::HashMap;

pub struct Config {
    from: String,
    replyto: String,
    cc: Vec<String>,
    subject: String,
    version: String,
    recipients: Vec<Recipient>,
}

#[derive(Debug)]
pub struct Recipient {
    email: String,
    names: Vec<String>,
    data: HashMap<String, String>,
}

impl PartialEq for Recipient {
    fn eq(&self, other: &Self) -> bool {
        self.email == other.email && self.names == other.names && self.data == other.data
    }
}

impl ToString for Recipient {
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

pub fn parse(_cfg: &ini::Ini) -> Result<Config, String> {
    return Err(String::from("not implemented"));
}

fn parse_general(_cfg: &ini::Ini) -> Result<Config, String> {
    return Err(String::from("not implemented"));
}

fn check_email(email: &str) -> bool {
    let re = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
    re.is_match(email.to_string().trim())
}

fn parse_recipient_data(rdata: &Vec<&str>) -> Result<HashMap<String, String>, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    for rd in rdata.iter() {
        // split the data, example: "Cc:-+inc@gg.org"
        let data: Vec<&str> = rd.split(":-").map(|w| w.trim()).collect();
        let key = data[0];
        let val = data[1];
        if key.len() == 0 && val.len() == 0 {
            continue;
        }
        if key.len() == 0 {
            return Err(String::from(format!("empty key for data ({})", val)));
        }
        if val.len() == 0 {
            return Err(String::from(format!("empty value for key ({})", key)));
        }
        result.insert(key.to_string(), val.to_string());
    }
    Ok(result)
}

fn parse_recipients(cfg: &ini::Ini) -> Result<Vec<Recipient>, String> {
    let mut result: Vec<Recipient> = Vec::new();
    let section = cfg.section(Some(String::from("recipients"))).unwrap();
    let mut keys: Vec<&String> = section.keys().collect();
    keys.sort();
    for key in keys {
        let val = section.get(key).unwrap();
        if !check_email(key) {
            return Err(format!("invalid email: {}", key));
        }
        // split recipient data, example:
        // John Doe Jr.|ORG:-EFF|TITLE:-PhD|Cc:-bl@kf.io,info@ex.org
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
            Err(msg) => {
                return Err(format!(
                    "invalid recipient data for email: {} ({})",
                    key, msg
                ))
            }
        }
    }
    return Ok(result);
}

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

    #[test]
    fn parse_recipients_with_happy_path() {
        let file = r#"
[general]
From=abc@def.com
# this is a comment
Cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'Cc' setting below *redefines* the global 'Cc' value above
jd@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|Cc:-bl@kf.io,info@ex.org
mm@gmail.com=Mickey Mouse|ORG:-Disney   # trailing comment!!
# The 'Cc' setting below *adds* to the global 'Cc' value above
daisy@example.com=Daisy Lila|ORG:-NASA|TITLE:-Dr.|Cc:-+inc@gg.org"#;
        let cfg = prep_config(file).expect("Failed to set up config");
        let mut expected = Vec::new();
        expected.push(Recipient {
            email: String::from("daisy@example.com"),
            names: vec![String::from("Daisy"), String::from("Lila")],
            data: vec![
                (String::from("ORG"), String::from("NASA")),
                (String::from("TITLE"), String::from("Dr.")),
                (String::from("Cc"), String::from("+inc@gg.org")),
            ]
            .into_iter()
            .collect(),
        });
        expected.push(Recipient {
            email: String::from("jd@example.com"),
            names: vec![
                String::from("John"),
                String::from("Doe"),
                String::from("Jr."),
            ],
            data: vec![
                (String::from("ORG"), String::from("EFF")),
                (String::from("TITLE"), String::from("PhD")),
                (String::from("Cc"), String::from("bl@kf.io,info@ex.org")),
            ]
            .into_iter()
            .collect(),
        });
        expected.push(Recipient {
            email: String::from("mm@gmail.com"),
            names: vec![String::from("Mickey"), String::from("Mouse")],
            data: vec![(String::from("ORG"), String::from("Disney"))]
                .into_iter()
                .collect(),
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
            names: vec![
                String::from("John"),
                String::from("Doe"),
                String::from("Jr."),
            ],
            data: vec![
                (String::from("ORG"), String::from("EFF")),
                (String::from("TITLE"), String::from("PhD")),
                (String::from("Cc"), String::from("bl@kf.io,info@ex.org")),
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!("email: jd@example.com, names: John, Doe, Jr., data: Cc => bl@kf.io,info@ex.org, ORG => EFF, TITLE => PhD", r.to_string());
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
        assert_eq!(true, check_email("        abx+alias@yajo.co.uk        "));
    }

    #[test]
    fn check_email_with_failure() {
        assert_eq!(false, check_email("@yajo.co.uk"));
    }

    #[test]
    fn check_email_with_whitespace_failure() {
        assert_eq!(false, check_email("     @yajo.co.uk"));
        assert_eq!(false, check_email("hello@   .uk  "));
        assert_eq!(false, check_email("hello@"));
        assert_eq!(false, check_email("@"));
        assert_eq!(false, check_email("hello@       "));
    }

    #[test]
    fn parse_recipients_with_invalid_email() {
        let file = r#"
[general]
From=abc@def.com
# this is a comment
Cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'Cc' setting below *redefines* the global 'Cc' value above
@example.com=John Doe Jr.|ORG:-EFF|TITLE:-PhD|Cc:-bl@kf.io,info@ex.org"#;
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
Cc=weirdo@nsb.gov, cc@example.com
[recipients]
# The 'Cc' setting below *redefines* the global 'Cc' value above
a@example.com="#;
        let cfg = prep_config(file).expect("Failed to set up config");
        let expected = Err(String::from("invalid data for email: a@example.com"));
        assert_eq!(expected, parse_recipients(&cfg));
    }

    #[test]
    fn parse_recipient_data_happy_case() {
        let expected: HashMap<String, String> = vec![
            (String::from("ORG"), String::from("EFF")),
            (String::from("TITLE"), String::from("PhD")),
            (String::from("Cc"), String::from("bl@kf.io,info@ex.org")),
        ]
        .into_iter()
        .collect();
        let mut args: Vec<&str> =
            "jd@example.com=John Doe Jr.|ORG:-EFF|   TITLE:-PhD|Cc:-         bl@kf.io,info@ex.org"
                .split("|")
                .collect();

        args.remove(0);
        assert_eq!(Ok(expected), parse_recipient_data(&args));
    }

    #[test]
    fn parse_recipient_data_happy_case_with_empty_key_and_value() {
        let expected: HashMap<String, String> = vec![
            (String::from("ORG"), String::from("EFF")),
            (String::from("Cc"), String::from("bl@kf.io,info@ex.org")),
        ]
        .into_iter()
        .collect();
        let mut args: Vec<&str> =
            "jd@example.com=John Doe Jr.|ORG:-EFF|:-|Cc:-         bl@kf.io,info@ex.org"
                .split("|")
                .collect();

        args.remove(0);
        assert_eq!(Ok(expected), parse_recipient_data(&args));
    }

    #[test]
    fn parse_recipient_data_failure_case_with_empty_key() {
        let mut args: Vec<&str> =
            "jd@example.com=John Doe Jr.|:-EFF|TITLE:-PhD|Cc:-bl@kf.io,info@ex.org"
                .split("|")
                .collect();

        args.remove(0);
        assert_eq!(
            Err(String::from("empty key for data (EFF)")),
            parse_recipient_data(&args)
        );
    }

    #[test]
    fn parse_recipient_data_failure_case_with_empty_value() {
        let mut args: Vec<&str> =
            "jd@example.com=John Doe Jr.|ORG:-    |TITLE:-PhD|Cc:-bl@kf.io,info@ex.org"
                .split("|")
                .collect();

        args.remove(0);
        assert_eq!(
            Err(String::from("empty value for key (ORG)")),
            parse_recipient_data(&args)
        );
    }
}
