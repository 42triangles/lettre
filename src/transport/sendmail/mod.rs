//! The sendmail transport sends the email using the local sendmail command.
//!
//! #### Sendmail Transport
//!
//! The sendmail transport sends the email using the local sendmail command.
//!
//! ```rust,no_run
//! # #[cfg(feature = "sendmail-transport")]
//! # {
//! use lettre::{Message, Envelope, Transport, SendmailTransport};
//!
//! let email = Message::builder()
//!     .from("NoBody <nobody@domain.tld>".parse().unwrap())
//!     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
//!     .to("Hei <hei@domain.tld>".parse().unwrap())
//!     .subject("Happy new year")
//!     .body("Be happy!")
//!     .unwrap();
//!
//! let sender = SendmailTransport::new();
//! let result = sender.send(&email);
//! assert!(result.is_ok());
//! # }
//! ```

use crate::{transport::sendmail::error::SendmailResult, Envelope, Transport};
use std::{
    convert::AsRef,
    io::prelude::*,
    process::{Command, Stdio},
};

pub mod error;

/// Sends an email using the `sendmail` command
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendmailTransport {
    command: String,
}

impl SendmailTransport {
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> SendmailTransport {
        SendmailTransport {
            command: "/usr/sbin/sendmail".to_string(),
        }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<String>>(command: S) -> SendmailTransport {
        SendmailTransport {
            command: command.into(),
        }
    }
}

impl<'a> Transport<'a> for SendmailTransport {
    type Result = SendmailResult;

    fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Self::Result {
        // Spawn the sendmail command
        let mut process = Command::new(&self.command)
            .arg("-i")
            .arg("-f")
            .arg(envelope.from().map(|f| f.as_ref()).unwrap_or("\"\""))
            .args(envelope.to())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        process.stdin.as_mut().unwrap().write_all(email)?;
        let output = process.wait_with_output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(error::Error::Client(String::from_utf8(output.stderr)?))
        }
    }
}
