// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contains mixed-up tools for SMTP
//!
//! TODO : Clean up and split this file

use std::io::net::ip::Port;
use std::string::String;

/// Default SMTP port
pub static SMTP_PORT: Port = 25;

// Maximum length of an SMTP command line
//pub static MAX_SMTP_LINE_LENGTH: uint = 1034;

/// The word separator for SMTP transactions
pub static SP: &'static str = " ";

/// The line ending for SMTP transactions
pub static CRLF: &'static str = "\r\n";
pub static CR: &'static str = "\r";
pub static LF: &'static str = "\n";

/// Adds quotes to emails if needed
pub fn quote_email_address(address: String) -> String {
    match address.len() {
        0 ... 1 => format!("<{:s}>", address),
        _   => match (address.as_slice().slice_to(1),
                      address.as_slice().slice_from(address.len() - 1)) {
                   ("<", ">") => address,
                   _          => format!("<{:s}>", address)
               }
    }
}

/// Removes quotes from emails if needed
pub fn unquote_email_address(address: String) -> String {
    match address.len() {
        0 ... 1 => address,
        _    => match (address.as_slice().slice_to(1),
                       address.as_slice().slice_from(address.len() - 1)) {
                    ("<", ">") => address.as_slice().slice(1, address.len() - 1).to_string(),
                    _          => address
                }
    }
}

/// Removes the trailing line return at the end of a string
pub fn remove_trailing_crlf(string: String) -> String {
    if string.len() > 1 && string.as_slice().slice_from(string.len() - 2) == CRLF {
        string.as_slice().slice_to(string.len() - 2).to_string()
    } else if string.len() > 0 && string.as_slice().slice_from(string.len() - 1) == "\r" {
        string.as_slice().slice_to(string.len() - 1).to_string()
    } else {
        string
    }
}

/// Returns the first word of a string, or the string if it contains no space
pub fn get_first_word(string: String) -> String {
    string.as_slice().split_str(CRLF).next().unwrap().splitn(1, ' ').next().unwrap().to_string()
}

/// Returns the string replacing all the CRLF with "<CRLF>"
#[inline]
pub fn escape_crlf(string: String) -> String {
    string.replace(CRLF, "<CR><LF>")
}

/// Returns the string after adding a dot at the beginning of each line starting with a dot
/// Reference : https://tools.ietf.org/html/rfc5321#page-62 (4.5.2. Transparency)
#[inline]
pub fn escape_dot(string: String) -> String {
    if string.len() > 0 && string.chars().next().unwrap() == '.' {
        format!(".{}", string)
    } else {
        string
    }.replace(format!("{}.", CR).as_slice(), format!("{}..", CR).as_slice())
     .replace(format!("{}.", LF).as_slice(), format!("{}..", LF).as_slice())
}

#[cfg(test)]
mod test {
    use super::{CRLF, CR, LF};

    #[test]
    fn test_quote_email_address() {
        assert_eq!(super::quote_email_address("address".to_string()), "<address>".to_string());
        assert_eq!(super::quote_email_address("<address>".to_string()), "<address>".to_string());
        assert_eq!(super::quote_email_address("a".to_string()), "<a>".to_string());
        assert_eq!(super::quote_email_address("".to_string()), "<>".to_string());
    }

    #[test]
    fn test_unquote_email_address() {
        assert_eq!(super::unquote_email_address("<address>".to_string()), "address".to_string());
        assert_eq!(super::unquote_email_address("address".to_string()), "address".to_string());
        assert_eq!(super::unquote_email_address("<address".to_string()), "<address".to_string());
        assert_eq!(super::unquote_email_address("<>".to_string()), "".to_string());
        assert_eq!(super::unquote_email_address("a".to_string()), "a".to_string());
        assert_eq!(super::unquote_email_address("".to_string()), "".to_string());
    }

    #[test]
    fn test_remove_trailing_crlf() {
        assert_eq!(super::remove_trailing_crlf("word".to_string()), "word".to_string());
        assert_eq!(super::remove_trailing_crlf("word\r\n".to_string()), "word".to_string());
        assert_eq!(super::remove_trailing_crlf("word\r\n ".to_string()), "word\r\n ".to_string());
        assert_eq!(super::remove_trailing_crlf("word\r".to_string()), "word".to_string());
        assert_eq!(super::remove_trailing_crlf("\r\n".to_string()), "".to_string());
        assert_eq!(super::remove_trailing_crlf("\r".to_string()), "".to_string());
        assert_eq!(super::remove_trailing_crlf("a".to_string()), "a".to_string());
        assert_eq!(super::remove_trailing_crlf("".to_string()), "".to_string());
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(super::get_first_word("first word".to_string()), "first".to_string());
        assert_eq!(super::get_first_word("first word\r\ntest".to_string()), "first".to_string());
        assert_eq!(super::get_first_word("first".to_string()), "first".to_string());
        assert_eq!(super::get_first_word("".to_string()), "".to_string());
        assert_eq!(super::get_first_word("\r\n".to_string()), "".to_string());
        assert_eq!(super::get_first_word("a\r\n".to_string()), "a".to_string());
        // Manage cases of empty line, spaces at the beginning
        //assert_eq!(super::get_first_word(" a".to_string()), "a".to_string());
        //assert_eq!(super::get_first_word("\r\n a".to_string()), "a".to_string());
        assert_eq!(super::get_first_word(" \r\n".to_string()), "".to_string());
        assert_eq!(super::get_first_word("\r\n ".to_string()), "".to_string());
    }

    #[test]
    fn test_escape_crlf() {
        assert_eq!(super::escape_crlf(format!("{}", CRLF)), "<CR><LF>".to_string());
        assert_eq!(super::escape_crlf(format!("EHLO my_name{}", CRLF)), "EHLO my_name<CR><LF>".to_string());
        assert_eq!(super::escape_crlf(format!("EHLO my_name{}SIZE 42{}", CRLF, CRLF)), "EHLO my_name<CR><LF>SIZE 42<CR><LF>".to_string());
    }

    #[test]
    fn test_escape_dot() {
        assert_eq!(super::escape_dot(".test".to_string()), "..test".to_string());
        assert_eq!(super::escape_dot(format!("{}.{}.{}", CR, LF, CRLF)), format!("{}..{}..{}", CR, LF, CRLF));
        assert_eq!(super::escape_dot(format!("{}.{}.{}", CRLF, CRLF, CRLF)), format!("{}..{}..{}", CRLF, CRLF, CRLF));
        assert_eq!(super::escape_dot(format!("test{}.test{}", CRLF, CRLF)), format!("test{}..test{}", CRLF, CRLF));
        assert_eq!(super::escape_dot(format!("test{}.{}test", CRLF, CRLF)), format!("test{}..{}test", CRLF, CRLF));
    }
}
