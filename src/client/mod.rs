// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! SMTP client

use std::string::String;
use std::error::FromError;
use std::net::TcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::{BufRead, BufStream, Read, Write};

use serialize::base64::{self, ToBase64, FromBase64};
use serialize::hex::ToHex;
use crypto::hmac::Hmac;
use crypto::md5::Md5;
use crypto::mac::Mac;

use tools::{NUL, CRLF, MESSAGE_ENDING};
use tools::{escape_dot, escape_crlf};
use response::{Response, Severity, Category};
use error::SmtpResult;
use client::connecter::Connecter;

pub mod connecter;

/// Structure that implements the SMTP client
pub struct Client<S = TcpStream> {
    /// TCP stream between client and server
    /// Value is None before connection
    stream: Option<BufStream<S>>,
    /// Socket we are connecting to
    server_addr: SocketAddr,

}

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => return_err!(err, $client),
        }
    })
);

macro_rules! return_err (
    ($err: expr, $client: ident) => ({
        return Err(FromError::from_error($err))
    })
);

macro_rules! check_response (
    ($result: ident) => ({
        match $result {
            Ok(response) => {
                match response.is_positive() {
                    true => Ok(response),
                    false => Err(FromError::from_error(response)),
                }
            },
            Err(_) => $result,
        }
    })
);

impl<S = TcpStream> Client<S> {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Client`
    pub fn new<A: ToSocketAddrs>(addr: A) -> Client<S> {
        Client{
            stream: None,
            server_addr: addr.to_socket_addrs().ok().expect("could not parse server address").next().unwrap(),
        }
    }
}

impl<S: Connecter + Write + Read = TcpStream> Client<S> {
    /// Closes the SMTP transaction if possible
    pub fn close(&mut self) {
        let _ = self.quit();
    }

    /// Connects to the configured server
    pub fn connect(&mut self) -> SmtpResult {
        // Connect should not be called when the client is already connected
        if self.stream.is_some() {
            return_err!("The connection is already established", self);
        }

        // Try to connect
        self.stream = Some(BufStream::new(try!(Connecter::connect(&self.server_addr))));

        self.get_reply()
    }

    /// Checks if the server is connected using the NOOP SMTP command
    pub fn is_connected(&mut self) -> bool {
        self.noop().is_ok()
    }

    /// Sends an SMTP command
    pub fn command(&mut self, command: &str) -> SmtpResult {
        self.send_server(command, CRLF)
    }

    /// Send a HELO command and fills `server_info`
    pub fn helo(&mut self, hostname: &str) -> SmtpResult {
        self.command(format!("HELO {}", hostname).as_slice())
    }

    /// Sends a EHLO command and fills `server_info`
    pub fn ehlo(&mut self, hostname: &str) -> SmtpResult {
        self.command(format!("EHLO {}", hostname).as_slice())
    }

    /// Sends a MAIL command
    pub fn mail(&mut self, address: &str, options: Option<&str>) -> SmtpResult {
        match options {
            Some(ref options) => self.command(format!("MAIL FROM:<{}> {}", address, options).as_slice()),
            None => self.command(format!("MAIL FROM:<{}>", address).as_slice()),
        }
    }

    /// Sends a RCPT command
    pub fn rcpt(&mut self, address: &str) -> SmtpResult {
        self.command(format!("RCPT TO:<{}>", address).as_slice())
    }

    /// Sends a DATA command
    pub fn data(&mut self) -> SmtpResult {
        self.command("DATA")
    }

    /// Sends a QUIT command
    pub fn quit(&mut self) -> SmtpResult {
        self.command("QUIT")
    }

    /// Sends a NOOP command
    pub fn noop(&mut self) -> SmtpResult {
        self.command("NOOP")
    }

    /// Sends a HELP command
    pub fn help(&mut self, argument: Option<&str>) -> SmtpResult {
        match argument {
            Some(ref argument) => self.command(format!("HELP {}", argument).as_slice()),
            None => self.command("HELP"),
        }
    }

    /// Sends a VRFY command
    pub fn vrfy(&mut self, address: &str) -> SmtpResult {
        self.command(format!("VRFY {}", address).as_slice())
    }

    /// Sends a EXPN command
    pub fn expn(&mut self, address: &str) -> SmtpResult {
        self.command(format!("EXPN {}", address).as_slice())
    }

    /// Sends a RSET command
    pub fn rset(&mut self) -> SmtpResult {
        self.command("RSET")
    }

    /// Sends an AUTH command with PLAIN mecanism
    pub fn auth_plain(&mut self, username: &str, password: &str) -> SmtpResult {
        let auth_string = format!("{}{}{}{}", NUL, username, NUL, password);
        self.command(format!("AUTH PLAIN {}", auth_string.as_bytes().to_base64(base64::STANDARD)).as_slice())
    }

    /// Sends an AUTH command with CRAM-MD5 mecanism
    pub fn auth_cram_md5(&mut self, username: &str, password: &str) -> SmtpResult {
        let encoded_challenge = try_smtp!(self.command("AUTH CRAM-MD5"), self).first_word().expect("No challenge");
        // TODO manage errors
        let challenge = encoded_challenge.from_base64().unwrap();

        let mut hmac = Hmac::new(Md5::new(), password.as_bytes());
        hmac.input(challenge.as_slice());

        let auth_string = format!("{} {}", username, hmac.result().code().to_hex());

        self.command(format!("AUTH CRAM-MD5 {}", auth_string.as_bytes().to_base64(base64::STANDARD)).as_slice())
    }

    /// Sends the message content and close
    pub fn message(&mut self, message_content: &str) -> SmtpResult {
        self.send_server(escape_dot(message_content).as_slice(), MESSAGE_ENDING)
    }

    /// Sends a string to the server and gets the response
    fn send_server(&mut self, string: &str, end: &str) -> SmtpResult {
        if self.stream.is_none() {
            return Err(FromError::from_error("Connection closed"));
        }

        try!(write!(self.stream.as_mut().unwrap(), "{}{}", string, end));
        try!(self.stream.as_mut().unwrap().flush());

        debug!("Wrote: {}", escape_crlf(string));

        self.get_reply()
    }

    /// Gets the SMTP response
    fn get_reply(&mut self) -> SmtpResult {
        let mut line = String::new();
        try!(self.stream.as_mut().unwrap().read_line(&mut line));

        // If the string is too short to be a response code
        if line.len() < 3 {
            return Err(FromError::from_error("Could not parse reply code, line too short"));
        }

        let (severity, category, detail) =  match (line[0..1].parse::<Severity>(), line[1..2].parse::<Category>(), line[2..3].parse::<u8>()) {
            (Ok(severity), Ok(category), Ok(detail)) => (severity, category, detail),
            _ => return Err(FromError::from_error("Could not parse reply code")),
        };

        let mut message = Vec::new();

        // 3 chars for code + space + CRLF
        while line.len() > 6 {
            let end = line.len() - 2;
            message.push(line[4..end].to_string());
            if line.as_bytes()[3] == '-' as u8 {
                line.clear();
                try!(self.stream.as_mut().unwrap().read_line(&mut line));
            } else {
                line.clear();
            }
        }

        let response = Response::new(severity, category, detail, message);

        match response.is_positive() {
            true => Ok(response),
            false => Err(FromError::from_error(response)),
        }
    }
}
