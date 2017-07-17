extern crate lettre;

use lettre::{EmailAddress, EmailTransport, SendableEmail, SimpleSendableEmail};

use lettre::file::FileEmailTransport;
use std::env::temp_dir;
use std::fs::File;
use std::fs::remove_file;
use std::io::Read;

#[test]
fn file_transport() {
    let mut sender = FileEmailTransport::new(temp_dir());
    let email = SimpleSendableEmail::new(
        EmailAddress::new("user@localhost".to_string()),
        vec![EmailAddress::new("root@localhost".to_string())],
        "file_id".to_string(),
        "Hello file".to_string(),
    );
    let result = sender.send(email.clone());
    assert!(result.is_ok());

    let message_id = email.message_id();
    let file = format!("{}/{}.txt", temp_dir().to_str().unwrap(), message_id);
    let mut f = File::open(file.clone()).unwrap();
    let mut buffer = String::new();
    let _ = f.read_to_string(&mut buffer);

    assert_eq!(
        buffer,
        "{\"to\":[\"root@localhost\"],\"from\":\"user@localhost\",\
        \"message_id\":\"file_id\",\"message\":\"Hello file\"}"
    );

    remove_file(file).unwrap();
}
