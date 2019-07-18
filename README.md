# Rust mailing tool (rmt)

Please note: this is still very much work in progress..

## What is it?
`rmt` is a simple utility that allows the automated sending of emails using a configuration file and a template for the email body. It is written in [rust](https://www.rust-lang.org/) hence the `r` in `rmt`.

## Usage

### Configuration and template files

`rmt` uses a template (for the emails to be sent) and a config file (specifying the recipients etc.). The following commands will generate a sample config and template file respectively:

    $ rmt sample config > /tmp/sc.ini
    $ rmt sample template > /tmp/st.eml

Adjust these as needed to get going.
