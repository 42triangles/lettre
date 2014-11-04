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
use std::str::replace;

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
pub fn quote_email_address(address: &str) -> String {
    match address.len() {
        0 ... 1 => format!("<{:s}>", address),
        _   => match (address.slice_to(1),
                      address.slice_from(address.len() - 1)) {
                   ("<", ">") => address.to_string(),
                   _          => format!("<{:s}>", address)
               }
    }
}

/// Removes quotes from emails if needed
pub fn unquote_email_address(address: &str) -> &str {
    match address.len() {
        0 ... 1 => address,
        _    => match (address.slice_to(1),
                       address.slice_from(address.len() - 1)) {
                    ("<", ">") => address.slice(1, address.len() - 1),
                    _          => address
                }
    }
}

/// Removes the trailing line return at the end of a string
pub fn remove_trailing_crlf(string: &str) -> &str {
    if string.ends_with(CRLF) {
        string.slice_to(string.len() - 2)
    } else if string.ends_with(CR) {
        string.slice_to(string.len() - 1)
    } else {
        string
    }
}

/// Returns the first word of a string, or the string if it contains no space
pub fn get_first_word(string: &str) -> &str {
    string.split_str(CRLF).next().unwrap().splitn(1, ' ').next().unwrap()
}

/// Returns the string replacing all the CRLF with "<CRLF>"
#[inline]
pub fn escape_crlf(string: &str) -> String {
    replace(string, CRLF, "<CR><LF>")
}

/// Returns the string after adding a dot at the beginning of each line starting with a dot
/// Reference : https://tools.ietf.org/html/rfc5321#page-62 (4.5.2. Transparency)
#[inline]
pub fn escape_dot(string: &str) -> String {
    if string.starts_with(".") {
        format!(".{}", string)
    } else {
        string.to_string()
    }.replace(format!("{}.", CR).as_slice(), format!("{}..", CR).as_slice())
     .replace(format!("{}.", LF).as_slice(), format!("{}..", LF).as_slice())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_quote_email_address() {
        assert_eq!(super::quote_email_address("address").as_slice(), "<address>");
        assert_eq!(super::quote_email_address("<address>").as_slice(), "<address>");
        assert_eq!(super::quote_email_address("a").as_slice(), "<a>");
        assert_eq!(super::quote_email_address("").as_slice(), "<>");
    }

    #[test]
    fn test_unquote_email_address() {
        assert_eq!(super::unquote_email_address("<address>"), "address");
        assert_eq!(super::unquote_email_address("address"), "address");
        assert_eq!(super::unquote_email_address("<address"), "<address");
        assert_eq!(super::unquote_email_address("<>"), "");
        assert_eq!(super::unquote_email_address("a"), "a");
        assert_eq!(super::unquote_email_address(""), "");
    }

    #[test]
    fn test_remove_trailing_crlf() {
        assert_eq!(super::remove_trailing_crlf("word"), "word");
        assert_eq!(super::remove_trailing_crlf("word\r\n"), "word");
        assert_eq!(super::remove_trailing_crlf("word\r\n "), "word\r\n ");
        assert_eq!(super::remove_trailing_crlf("word\r"), "word");
        assert_eq!(super::remove_trailing_crlf("\r\n"), "");
        assert_eq!(super::remove_trailing_crlf("\r"), "");
        assert_eq!(super::remove_trailing_crlf("a"), "a");
        assert_eq!(super::remove_trailing_crlf(""), "");
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(super::get_first_word("first word"), "first");
        assert_eq!(super::get_first_word("first word\r\ntest"), "first");
        assert_eq!(super::get_first_word("first"), "first");
        assert_eq!(super::get_first_word(""), "");
        assert_eq!(super::get_first_word("\r\n"), "");
        assert_eq!(super::get_first_word("a\r\n"), "a");
        // Manage cases of empty line, spaces at the beginning
        //assert_eq!(super::get_first_word(" a"), "a");
        //assert_eq!(super::get_first_word("\r\n a"), "a");
        assert_eq!(super::get_first_word(" \r\n"), "");
        assert_eq!(super::get_first_word("\r\n "), "");
    }

    #[test]
    fn test_escape_crlf() {
        assert_eq!(super::escape_crlf("\r\n").as_slice(), "<CR><LF>");
        assert_eq!(super::escape_crlf("EHLO my_name\r\n").as_slice(), "EHLO my_name<CR><LF>");
        assert_eq!(super::escape_crlf("EHLO my_name\r\nSIZE 42\r\n").as_slice(), "EHLO my_name<CR><LF>SIZE 42<CR><LF>");
    }

    #[test]
    fn test_escape_dot() {
        assert_eq!(super::escape_dot(".test").as_slice(), "..test");
        assert_eq!(super::escape_dot("\r.\n.\r\n").as_slice(), "\r..\n..\r\n");
        assert_eq!(super::escape_dot("test\r\n.test\r\n").as_slice(), "test\r\n..test\r\n");
        assert_eq!(super::escape_dot("test\r\n.\r\ntest").as_slice(), "test\r\n..\r\ntest");
    }
}
