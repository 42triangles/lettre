// Copyright 2014 Alexis Mousset. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(phase)] #[phase(plugin, link)] extern crate log;

extern crate smtp;
extern crate getopts;

use std::io::stdin;
use std::io::net::tcp::TcpStream;
use std::string::String;
use std::io::net::ip::Port;
use std::os;
use getopts::{optopt, optflag, getopts, OptGroup, usage};

use smtp::client::Client;
use smtp::error::SmtpResult;

fn sendmail(source_address: &str, recipient_addresses: Vec<&str>, message: &str,
        server: Option<&str>, port: Option<Port>, my_hostname: Option<&str>) -> SmtpResult {
    let mut email_client: Client<TcpStream> =
        Client::new(
            server,
            port,
            my_hostname
        );
    email_client.send_mail::<TcpStream>(
            source_address,
            recipient_addresses,
            message
    )
}

fn print_usage(description: String, _opts: &[OptGroup]) {
    println!("{}", usage(description.as_slice(), _opts));
}

fn main() {
    let args = os::args();

    let mut args_string = Vec::new();
    for arg in args.iter() {
        args_string.push(arg.clone());
    };

    let program = args[0].clone();
    let description = format!("Usage: {0} [options...] recipients\n\n\
                               This program reads a message on standard input until it reaches EOF,\
                               then tries to send it using the given paramters.\n\n\
                               Example: {0} -r user@example.org user@example.com < message.txt", program);

    let opts = [
        optopt("r", "reverse-path", "set the sender address", "FROM_ADDRESS"),
        optopt("p", "port", "set the port to use, default is 25", "PORT"),
        optopt("s", "server", "set the server to use, default is localhost", "SERVER"),
        optopt("m", "my-hostname", "set the hostname used by the client", "MY_HOSTNAME"),
        optflag("h", "help", "print this help menu"),
        optflag("v", "verbose", "display the transaction details"),
    ];

    let matches = match getopts(args_string.tail(), opts) {
        Ok(m) => m,
        Err(f) => panic!("{}", f)
    };

    if matches.opt_present("h") {
        print_usage(description, opts);
        return;
    }

    if !matches.opt_present("r") {
        println!("The sender option is required");
        print_usage(program, opts);
        return;
    }

    let recipients_str: &str = if !matches.free.is_empty() {
        matches.free[0].as_slice()
    } else {
        print_usage(description, opts);
        return;
    };

    let mut recipients = Vec::new();
    for recipient in recipients_str.split(' ') {
        recipients.push(recipient);
    }

    let mut message = String::new();
    for line in stdin().lines() {
        message.push_str(line.unwrap().as_slice());
    }

    match sendmail(
        // sender
        matches.opt_str("r").unwrap().as_slice(),
        // recipients
        recipients,
        // message content
        message.as_slice(),
        // server
        match matches.opt_str("s") {
            Some(ref server) => Some(server.as_slice()),
            None => None
        },
        // port
        match matches.opt_str("p") {
            Some(port) => from_str::<Port>(port.as_slice()),
            None       => None
        },
        // my hostname
        match matches.opt_str("m") {
            Some(ref my_hostname) => Some(my_hostname.as_slice()),
            None => None
        }
    )
    {
        Ok(..) => info!("Email sent successfully"),
        Err(error) => error!("{}", error)
    }
}
