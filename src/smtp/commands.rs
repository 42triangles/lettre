/*!
 * SMTP commands library
 *
 * RFC 5321 : http://tools.ietf.org/html/rfc5321#section-4.1
 */

use std::fmt;
use common::CRLF;

/*
 * HELO <SP> <domain> <CRLF>
 * MAIL <SP> FROM:<reverse-path> <CRLF>
 * RCPT <SP> TO:<forward-path> <CRLF>
 * DATA <CRLF>
 * RSET <CRLF>
 * SEND <SP> FROM:<reverse-path> <CRLF>
 * SOML <SP> FROM:<reverse-path> <CRLF>
 * SAML <SP> FROM:<reverse-path> <CRLF>
 * VRFY <SP> <string> <CRLF>
 * EXPN <SP> <string> <CRLF>
 * HELP [<SP> <string>] <CRLF>
 * NOOP <CRLF>
 * QUIT <CRLF>
 * TURN <CRLF>
 */

/// List of SMTP commands
#[deriving(Eq,Clone)]
pub enum Command {
    Hello,
    Ehello,
    Mail,
    Recipient,
    Data,
    Reset,
    SendMail,
    SendOrMail,
    SendAndMail,
    Verify,
    Expand,
    Help,
    Noop,
    Quit,
    /// Deprecated in RFC 5321
    Turn,
}

impl Command {
    /// Tell if the command accetps an string argument.
    pub fn takes_argument(&self) -> bool{
        match *self {
            Ehello        => true,
            Hello         => true,
            Mail          => true,
            Recipient     => true,
            Data          => false,
            Reset         => false,
            SendMail      => true,
            SendOrMail    => true,
            SendAndMail   => true,
            Verify        => true,
            Expand        => true,
            Help          => true,
            Noop          => false,
            Quit          => false,
            Turn          => false,
        }
    }

    /// Tell if an argument is needed by the command.
    pub fn needs_argument(&self) -> bool {
        match *self {
            Ehello        => true,
            Hello         => true,
            Mail          => true,
            Recipient     => true,
            Data          => false,
            Reset         => false,
            SendMail      => true,
            SendOrMail    => true,
            SendAndMail   => true,
            Verify        => true,
            Expand        => true,
            Help          => false,
            Noop          => false,
            Quit          => false,
            Turn          => false,
        }
    }
}

impl ToStr for Command {
    /// Get the name of a command.
    fn to_str(&self) -> ~str {
        match *self {
            Hello           => ~"HELO",
            Ehello          => ~"EHLO",
            Mail            => ~"MAIL",
            Recipient       => ~"RCPT",
            Data            => ~"DATA",
            Reset           => ~"RSET",
            SendMail        => ~"SEND",
            SendOrMail      => ~"SOML",
            SendAndMail     => ~"SAML",
            Verify          => ~"VRFY",
            Expand          => ~"EXPN",
            Help            => ~"HELP",
            Noop            => ~"NOOP",
            Quit            => ~"QUIT",
            Turn            => ~"TURN",
        }
    }
}

impl FromStr for Command {
    /// Get the Command from its name.
    fn from_str(command: &str) -> Option<Command> {
        if !command.is_ascii() {
            return None;
        }
        match command {
            "HELO" => Some(Hello),
            "EHLO" => Some(Ehello),
            "MAIL" => Some(Mail),
            "RCPT" => Some(Recipient),
            "DATA" => Some(Data),
            "RSET" => Some(Reset),
            "SEND" => Some(SendMail),
            "SOML" => Some(SendOrMail),
            "SAML" => Some(SendAndMail),
            "VRFY" => Some(Verify),
            "EXPN" => Some(Expand),
            "HELP" => Some(Help),
            "NOOP" => Some(Noop),
            "QUIT" => Some(Quit),
            "TURN" => Some(Turn),
            _      => None,
        }
    }
}

impl fmt::Default for Command {
    /// Format SMTP command display
    fn fmt(s: &Command, f: &mut fmt::Formatter) {
        f.buf.write(match *s {
            Ehello        => "EHLO".as_bytes(),
            Hello         => "HELO".as_bytes(),
            Mail          => "MAIL FROM:".as_bytes(),
            Recipient     => "RCPT TO:".as_bytes(),
            Data          => "DATA".as_bytes(),
            Reset         => "RSET".as_bytes(),
            SendMail      => "SEND TO:".as_bytes(),
            SendOrMail    => "SOML TO:".as_bytes(),
            SendAndMail   => "SAML TO:".as_bytes(),
            Verify        => "VRFY".as_bytes(),
            Expand        => "EXPN".as_bytes(),
            Help          => "HELP".as_bytes(),
            Noop          => "NOOP".as_bytes(),
            Quit          => "QUIT".as_bytes(),
            Turn          => "TURN".as_bytes()
        })
    }
}

/// Structure for a complete SMTP command, containing an optionnal string argument.
pub struct SmtpCommand {
    command: Command,
    argument: Option<~str>
}

impl SmtpCommand {
    /// Return a new structure from the name of the command and an optionnal argument.
    pub fn new(command_str: ~str, argument: Option<~str>) -> SmtpCommand {
        let command = match from_str::<Command>(command_str) {
            Some(x) => x,
            None    => fail!("Unrecognized SMTP command")
        };

        match (command.takes_argument(), command.needs_argument(), argument.clone()) {
            (true, true, None)      => fail!("Wrong SMTP syntax : argument needed"),
            (false, false, Some(x)) => fail!("Wrong SMTP syntax : {:s} not accepted", x),
            _                       => SmtpCommand {command: command, argument: argument}
        }
    }

    /// Return the formatted command, ready to be used in an SMTP session.
    pub fn get_formatted_command(&self) -> ~str {
        match (self.command.takes_argument(), self.command.needs_argument(), self.argument.clone()) {
            (true, _, Some(argument)) => format!("{} {}{}", self.command, argument, CRLF),
            (_, false, None)   => format!("{}{}", self.command, CRLF),
            _                  => fail!("Wrong SMTP syntax")
        }
    }
}

#[cfg(test)]
mod test {
    use super::SmtpCommand;

    #[test]
    fn test_command_parameters() {
        assert!((super::Help).takes_argument() == true);
        assert!((super::Reset).takes_argument() == false);
        assert!((super::Hello).needs_argument() == true);
    }

    #[test]
    fn test_get_simple_command() {
        assert!(SmtpCommand::new(~"TURN", None).get_formatted_command() == format!("TURN{}", ::common::CRLF));
    }

    #[test]
    fn test_get_argument_command() {
        assert!(SmtpCommand::new(~"EHLO", Some(~"example.example")).get_formatted_command() == format!("EHLO example.example{}", ::common::CRLF));
    }
}
