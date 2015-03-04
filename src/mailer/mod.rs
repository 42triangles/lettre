// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple email (very incomplete)

use std::fmt::{Display, Formatter, Result};

use time::{now, Tm};

use mailer::header::{ToHeader, Header};
use mailer::address::ToAddress;
use tools::CRLF;
use sendable_email::SendableEmail;

pub mod header;
pub mod address;

/// TODO
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct EmailBuilder {
    /// Email content
    content: Email,
    /// Date issued
    date_issued: bool,
}

/// Simple email representation
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Email {
    /// Array of headers
    headers: Vec<Header>,
    /// Message body
    body: String,
    /// The enveloppe recipients addresses
    to: Vec<String>,
    /// The enveloppe sender address
    from: Option<String>,
}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut formatted_headers = String::new();
        for header in self.headers.iter() {
            formatted_headers.push_str(format! ("{}", header) .as_slice());
            formatted_headers.push_str(CRLF);
        }
        write! (f, "{}{}{}", formatted_headers, CRLF, self.body)
    }
}

impl EmailBuilder {
    /// Creates a new empty email
    pub fn new() -> EmailBuilder {
        EmailBuilder {
            content: Email {
                headers: vec![],
                body: "".to_string(),
                to: vec![],
                from: None,
            },
            date_issued: false,
        }
    }

    /// Sets the email body
    pub fn body(mut self, body: &str) -> EmailBuilder {
        self.content.body = body.to_string();
        self
    }

    /// Add a generic header
    pub fn add_header<A: ToHeader>(mut self, header: A) -> EmailBuilder {
        self.content.headers.push(header.to_header());
        self
    }

    /// Adds a `From` header and store the sender address
    pub fn from<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.from = Some(address.to_address().get_address());
        self.content.headers.push(
            Header::From(address.to_address())
        );
        self
    }

    /// Adds a `To` header and store the recipient address
    pub fn to<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.to.push(address.to_address().get_address());
        self.content.headers.push(
            Header::To(address.to_address())
        );
        self
    }

    /// Adds a `Cc` header and store the recipient address
    pub fn cc<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.to.push(address.to_address().get_address());
        self.content.headers.push(
            Header::Cc(address.to_address())
        );
        self
    }

    /// Adds a `Reply-To` header
    pub fn reply_to<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.headers.push(
            Header::ReplyTo(address.to_address())
        );
        self
    }

    /// Adds a `Sender` header
    pub fn sender<A: ToAddress>(mut self, address: A) -> EmailBuilder {
        self.content.headers.push(
            Header::Sender(address.to_address())
        );
        self
    }

    /// Adds a `Subject` header
    pub fn subject(mut self, subject: &str) -> EmailBuilder {
        self.content.headers.push(
            Header::Subject(subject.to_string())
        );
        self
    }

    /// Adds a `Date` header with the given date
    pub fn date(mut self, date: Tm) -> EmailBuilder {
        self.content.headers.push(
            Header::Date(date)
        );
        self.date_issued = true;
        self
    }

    /// Build the Email
    pub fn build(mut self) -> Email {
        if !self.date_issued {
            self.content.headers.push(
                Header::Date(now())
            );
        }
        self.content
    }
}

impl SendableEmail for Email {
    /// Return the to addresses, and fails if it is not set
    fn to_addresses(&self) -> Vec<String> {
        if self.to.is_empty() {
            panic!("The To field is empty")
        }
        self.to.clone()
    }

    /// Return the from address, and fails if it is not set
    fn from_address(&self) -> String {
        match self.from {
            Some(ref from_address) => from_address.clone(),
            None => panic!("The From field is empty"),
        }
    }

    fn message(&self) -> String {
        format! ("{}", self)
    }

    /// Adds a `Message-ID` header
    fn set_message_id(&mut self, string: String) {
        self.headers.push(
            Header::MessageId(string)
        );
    }
}

#[cfg(test)]
mod test {
    use super::{Email, EmailBuilder};
    use mailer::header::Header;

    #[test]
    fn test_new() {
        assert_eq!(
            EmailBuilder::new(),
            EmailBuilder{content: Email{headers: vec![], body: "".to_string(), to: vec![], from: None}, date_issued: false}
        )
    }

    #[test]
    fn test_body() {
        let email = EmailBuilder::new().body("test message");
        assert_eq!(
            email,
            EmailBuilder{content: Email {headers: vec![], body: "test message".to_string(), to: vec![], from: None}, date_issued: false}
        )
    }

    #[test]
    fn test_add_header() {
        let mut email = EmailBuilder::new()
                        .add_header(("X-My-Header", "value"));
        assert_eq!(
            email,
            EmailBuilder{
                content: Email {
                    headers: vec![Header::new("X-My-Header", "value")],
                    body: "".to_string(),
                    to: vec![],
                    from: None
                },
                date_issued: false,
            }
        );
        email = email.add_header(("X-My-Header-2", "value-2"));
        assert_eq!(
            email,
            EmailBuilder{
                content: Email {
                    headers: vec![Header::new("X-My-Header", "value"),
                                  Header::new("X-My-Header-2", "value-2")],
                    body: "".to_string(),
                    to: vec![],
                    from: None
                },
                date_issued: false,
            }
        );
        email = email.add_header(("X-My-Header-3", "value-3")).add_header(("X-My-Header-4", "value-4"));
        assert_eq!(
            email,
            EmailBuilder{
                content: Email {
                    headers: vec![Header::new("X-My-Header", "value"),
                                  Header::new("X-My-Header-2", "value-2"),
                                  Header::new("X-My-Header-3", "value-3"),
                                  Header::new("X-My-Header-4", "value-4")],
                    body: "".to_string(),
                    to: vec![],
                    from: None
                },
                date_issued: false,
            }
        );
    }

    // TODO test Email
}
