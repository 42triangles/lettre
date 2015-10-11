//! Sends an email using the client

use std::string::String;
use std::net::{SocketAddr, ToSocketAddrs};

use openssl::ssl::{SslMethod, SslContext};

use SMTP_PORT;
use extension::{Extension, ServerInfo};
use error::{SmtpResult, Error};
use email::SendableEmail;
use client::Client;
use authentication::Mecanism;

/// Contains client configuration
pub struct SenderBuilder {
    /// Maximum connection reuse
    ///
    /// Zero means no limitation
    connection_reuse_count_limit: u16,
    /// Enable connection reuse
    enable_connection_reuse: bool,
    /// Name sent during HELO or EHLO
    hello_name: String,
    /// Credentials
    credentials: Option<(String, String)>,
    /// Socket we are connecting to
    server_addr: SocketAddr,
    /// SSL contexyt to use
    ssl_context: Option<SslContext>,
    /// List of authentication mecanism, sorted by priority
    authentication_mecanisms: Vec<Mecanism>,
}

/// Builder for the SMTP Sender
impl SenderBuilder {
    /// Creates a new local SMTP client
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<SenderBuilder, Error> {
        let mut addresses = try!(addr.to_socket_addrs());

        match addresses.next() {
            Some(addr) => Ok(SenderBuilder {
                server_addr: addr,
                ssl_context: None,
                credentials: None,
                connection_reuse_count_limit: 100,
                enable_connection_reuse: false,
                hello_name: "localhost".to_string(),
                authentication_mecanisms: vec![Mecanism::CramMd5, Mecanism::Plain],
            }),
            None => Err(From::from("Could nor resolve hostname")),
        }
    }

    /// Creates a new local SMTP client to port 25
    pub fn localhost() -> Result<SenderBuilder, Error> {
        SenderBuilder::new(("localhost", SMTP_PORT))
    }

    /// Use STARTTLS with a specific context
    pub fn ssl_context(mut self, ssl_context: SslContext) -> SenderBuilder {
        self.ssl_context = Some(ssl_context);
        self
    }

    /// Require SSL/TLS using STARTTLS
    pub fn starttls(self) -> SenderBuilder {
        self.ssl_context(SslContext::new(SslMethod::Tlsv1).unwrap())
    }

    /// Set the name used during HELO or EHLO
    pub fn hello_name(mut self, name: &str) -> SenderBuilder {
        self.hello_name = name.to_string();
        self
    }

    /// Enable connection reuse
    pub fn enable_connection_reuse(mut self, enable: bool) -> SenderBuilder {
        self.enable_connection_reuse = enable;
        self
    }

    /// Set the maximum number of emails sent using one connection
    pub fn connection_reuse_count_limit(mut self, limit: u16) -> SenderBuilder {
        self.connection_reuse_count_limit = limit;
        self
    }

    /// Set the client credentials
    pub fn credentials(mut self, username: &str, password: &str) -> SenderBuilder {
        self.credentials = Some((username.to_string(), password.to_string()));
        self
    }

    /// Set the authentication mecanisms
    pub fn authentication_mecanisms(mut self, mecanisms: Vec<Mecanism>) -> SenderBuilder {
        self.authentication_mecanisms = mecanisms;
        self
    }

    /// Build the SMTP client
    ///
    /// It does not connects to the server, but only creates the `Sender`
    pub fn build(self) -> Sender {
        Sender::new(self)
    }
}

/// Represents the state of a client
#[derive(Debug)]
struct State {
    /// Panic state
    pub panic: bool,
    /// Connection reuse counter
    pub connection_reuse_count: u16,
}

/// Structure that implements the high level SMTP client
pub struct Sender {
    /// Information about the server
    /// Value is None before HELO/EHLO
    server_info: Option<ServerInfo>,
    /// Sender variable states
    state: State,
    /// Information about the client
    client_info: SenderBuilder,
    /// Low level client
    client: Client,
}

macro_rules! try_smtp (
    ($err: expr, $client: ident) => ({
        match $err {
            Ok(val) => val,
            Err(err) => {
                if !$client.state.panic {
                    $client.state.panic = true;
                    $client.reset();
                }
                return Err(err)
            },
        }
    })
);

impl Sender {
    /// Creates a new SMTP client
    ///
    /// It does not connects to the server, but only creates the `Sender`
    pub fn new(builder: SenderBuilder) -> Sender {

        let client = Client::new();

        Sender {
            client: client,
            server_info: None,
            client_info: builder,
            state: State {
                panic: false,
                connection_reuse_count: 0,
            },
        }
    }

    /// Reset the client state
    fn reset(&mut self) {
        // Close the SMTP transaction if needed
        self.close();

        // Reset the client state
        self.server_info = None;
        self.state.panic = false;
        self.state.connection_reuse_count = 0;
    }

    /// Closes the inner connection
    pub fn close(&mut self) {
        self.client.close();
    }

    /// Gets the EHLO response and updates server information
    pub fn get_ehlo(&mut self) -> SmtpResult {
        // Extended Hello
        let ehlo_response = try_smtp!(self.client.ehlo(&self.client_info.hello_name), self);

        self.server_info = Some(try_smtp!(ServerInfo::from_response(&ehlo_response), self));

        // Print server information
        debug!("server {}", self.server_info.as_ref().unwrap());

        Ok(ehlo_response)
    }

    /// Sends an email
    pub fn send<T: SendableEmail>(&mut self, email: T) -> SmtpResult {
        // Check if the connection is still available
        if self.state.connection_reuse_count > 0 {
            if !self.client.is_connected() {
                self.reset();
            }
        }

        // If there is a usable connection, test if the server answers and hello has been sent
        if self.state.connection_reuse_count == 0 {
            try!(self.client.connect(&self.client_info.server_addr));

            // Log the connection
            info!("connection established to {}", self.client_info.server_addr);

            try!(self.get_ehlo());

            if self.client_info.ssl_context.is_some() {
                try_smtp!(self.client.starttls(), self);

                try!(self.client.upgrade_tls_stream(self.client_info.ssl_context.as_ref().unwrap()));

                try!(self.get_ehlo());
            }

            if self.client_info.credentials.is_some() && self.state.connection_reuse_count == 0 {
                let (username, password) = self.client_info.credentials.clone().unwrap();

                let mut found = false;

                for mecanism in self.client_info.authentication_mecanisms.clone() {
                    if self.server_info.as_ref().unwrap().supports_auth_mecanism(mecanism) {
                        found = true;
                        let result = self.client.auth(mecanism, &username, &password);
                        try_smtp!(result, self);
                    }
                }

                if !found {
                    debug!("No supported authentication mecanisms available");
                }
            }
        }

        let current_message = try!(email.message_id().ok_or("Missing Message-ID"));
        let from_address = try!(email.from_address().ok_or("Missing From address"));
        let to_addresses = try!(email.to_addresses().ok_or("Missing To address"));
        let message = try!(email.message().ok_or("Missing message"));

        // Mail
        let mail_options = match self.server_info
                                     .as_ref()
                                     .unwrap()
                                     .supports_feature(&Extension::EightBitMime) {
            true => Some("BODY=8BITMIME"),
            false => None,
        };

        try_smtp!(self.client.mail(&from_address, mail_options), self);

        // Log the mail command
        info!("{}: from=<{}>", current_message, from_address);

        // Recipient
        for to_address in to_addresses.iter() {
            try_smtp!(self.client.rcpt(&to_address), self);
            // Log the rcpt command
            info!("{}: to=<{}>", current_message, to_address);
        }

        // Data
        try_smtp!(self.client.data(), self);

        // Message content
        let result = self.client.message(&message);

        if result.is_ok() {
            // Increment the connection reuse counter
            self.state.connection_reuse_count = self.state.connection_reuse_count + 1;

            // Log the message
            info!("{}: conn_use={}, size={}, status=sent ({})",
                  current_message,
                  self.state.connection_reuse_count,
                  message.len(),
                  result.as_ref()
                        .ok()
                        .unwrap()
                        .message()
                        .iter()
                        .next()
                        .unwrap_or(&"no response".to_string()));
        }

        // Test if we can reuse the existing connection
        if (!self.client_info.enable_connection_reuse) ||
           (self.state.connection_reuse_count >= self.client_info.connection_reuse_count_limit) {
            self.reset();
        }

        result
    }
}
