// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP response, containing a mandatory return code, and an optional text message

use std::str::FromStr;
use std::fmt::{Display, Formatter, Result};
use std::result::Result as RResult;

use self::Severity::*;
use self::Category::*;

/// First digit indicates severity
#[derive(PartialEq,Eq,Copy,Clone,Debug)]
pub enum Severity {
    /// 2yx
    PositiveCompletion,
    /// 3yz
    PositiveIntermediate,
    /// 4yz
    TransientNegativeCompletion,
    /// 5yz
    PermanentNegativeCompletion,
}

impl FromStr for Severity {
    type Err = &'static str;
    fn from_str(s: &str) -> RResult<Severity, &'static str> {
        match s {
            "2" => Ok(PositiveCompletion),
            "3" => Ok(PositiveIntermediate),
            "4" => Ok(TransientNegativeCompletion),
            "5" => Ok(PermanentNegativeCompletion),
            _ => Err("First digit must be between 2 and 5"),
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}",
            match *self {
                PositiveCompletion => 2,
                PositiveIntermediate => 3,
                TransientNegativeCompletion => 4,
                PermanentNegativeCompletion => 5,
            }
        )
    }
}

/// Second digit
#[derive(PartialEq,Eq,Copy,Clone,Debug)]
pub enum Category {
    /// x0z
    Syntax,
    /// x1z
    Information,
    /// x2z
    Connections,
    /// x3z
    Unspecified3,
    /// x4z
    Unspecified4,
    /// x5z
    MailSystem,
}

impl FromStr for Category {
    type Err = &'static str;
    fn from_str(s: &str) -> RResult<Category, &'static str> {
        match s {
            "0" => Ok(Syntax),
            "1" => Ok(Information),
            "2" => Ok(Connections),
            "3" => Ok(Unspecified3),
            "4" => Ok(Unspecified4),
            "5" => Ok(MailSystem),
            _ => Err("Second digit must be between 0 and 5"),
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}",
            match *self {
                Syntax => 0,
                Information => 1,
                Connections => 2,
                Unspecified3 => 3,
                Unspecified4 => 4,
                MailSystem => 5,
            }
        )
    }
}

/// Contains an SMTP reply, with separed code and message
///
/// The text message is optional, only the code is mandatory
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Response {
    /// First digit of the response code
    severity: Severity,
    /// Second digit of the response code
    category: Category,
    /// Third digit
    detail: u8,
    /// Server response string (optional)
    /// Handle multiline responses
    message: Vec<String>
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let code = self.code();
        for line in self.message[..-1].iter() {
            let _ = write!(f, "{}-{}",
                code,
                line
            );
        }
        write!(f, "{} {}",
            code,
            self.message[-1]
        )

    }
}

impl Response {
    /// Creates a new `Response`
    pub fn new(severity: Severity, category: Category, detail: u8, message: Vec<String>) -> Response {
        Response {
            severity: severity,
            category: category,
            detail: detail,
            message: message
        }
    }

    /// Tells if the response is positive
    pub fn is_positive(&self) -> bool {
        match self.severity {
            PositiveCompletion => true,
            PositiveIntermediate => true,
            _ => false,
        }
    }

    /// Returns the message
    pub fn message(&self) -> Vec<String> {
        self.message.clone()
    }

    /// Returns the severity (i.e. 1st digit)
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the category (i.e. 2nd digit)
    pub fn category(&self) -> Category {
        self.category
    }

    /// Returns the detail (i.e. 3rd digit)
    pub fn detail(&self) -> u8 {
        self.detail
    }

    /// Returns the reply code
    fn code(&self) -> String {
        format!("{}{}{}", self.severity, self.category, self.detail)
    }

    /// Checls code equality
    pub fn has_code(&self, code: u16) -> bool {
        self.code() == format!("{}", code)
    }

    /// Returns only the first word of the message if possible
    pub fn first_word(&self) -> Option<String> {
        match self.message.is_empty() {
            true => None,
            false => Some(self.message[0].words().next().unwrap().to_string())
        }

    }
}

#[cfg(test)]
mod test {
    use super::{Severity, Category, Response};

    #[test]
    fn test_severity_from_str() {
        assert_eq!("2".parse::<Severity>(), Ok(Severity::PositiveCompletion));
        assert_eq!("4".parse::<Severity>(), Ok(Severity::TransientNegativeCompletion));
        assert!("1".parse::<Severity>().is_err());
    }

    #[test]
    fn test_severity_fmt() {
        assert_eq!(format!("{}", Severity::PositiveCompletion).as_slice(), "2");
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!("2".parse::<Category>(), Ok(Category::Connections));
        assert_eq!("4".parse::<Category>(), Ok(Category::Unspecified4));
        assert!("6".parse::<Category>().is_err());
    }

    #[test]
    fn test_category_fmt() {
        assert_eq!(format!("{}", Category::Unspecified4).as_slice(), "4");
    }
}
