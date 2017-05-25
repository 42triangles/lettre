//! The file transport writes the emails to the given directory. The name of the file will be
//! `message_id.txt`.
//! It can be useful for testing purposes, or if you want to keep track of sent messages.
//!
//! ```rust
//! use std::env::temp_dir;
//!
//! use lettre::file::FileEmailTransport;
//! use lettre::{SimpleSendableEmail, EmailTransport};
//!
//! // Write to the local temp directory
//! let mut sender = FileEmailTransport::new(temp_dir());
//! let email = SimpleSendableEmail::new(
//!                 "user@localhost",
//!                 vec!["root@localhost"],
//!                 "message_id",
//!                 "Hello world"
//!             );
//!
//! let result = sender.send(email);
//! assert!(result.is_ok());
//! ```
//! Example result in `/tmp/b7c211bc-9811-45ce-8cd9-68eab575d695.txt`:
//!
//! ```text
//! b7c211bc-9811-45ce-8cd9-68eab575d695: from=<user@localhost> to=<root@localhost>
//! To: <root@localhost>
//! From: <user@localhost>
//! Subject: Hello
//! Date: Sat, 31 Oct 2015 13:42:19 +0100
//! Message-ID: <b7c211bc-9811-45ce-8cd9-68eab575d695.lettre@localhost>
//!
//! Hello World!
//! ```

use EmailTransport;
use SendableEmail;
use file::error::FileResult;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub mod error;

/// Writes the content and the envelope information to a file
#[derive(Debug)]
pub struct FileEmailTransport {
    path: PathBuf,
}

impl FileEmailTransport {
    /// Creates a new transport to the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> FileEmailTransport {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        FileEmailTransport { path: path_buf }
    }
}

impl EmailTransport<FileResult> for FileEmailTransport {
    fn send<T: SendableEmail>(&mut self, email: T) -> FileResult {
        let mut file = self.path.clone();
        file.push(format!("{}.txt", email.message_id()));

        let mut f = try!(File::create(file.as_path()));

        let log_line = format!("{}: from=<{}> to=<{}>\n",
                               email.message_id(),
                               email.from(),
                               email.to().join("> to=<"));

        try!(f.write_all(log_line.as_bytes()));
        try!(f.write_all(email.message().as_bytes()));

        info!("{} status=<written>", log_line);

        Ok(())
    }

    fn close(&mut self) {
        ()
    }
}
