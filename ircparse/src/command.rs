#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Privmsg {
        channel: &'a [u8],
        message: &'a [u8],
    },
    Unknown {
        command: &'a [u8],
        args: Vec<&'a [u8]>,
    },
}

impl<'a> Command<'a> {
    pub fn with_command_args(command: &'a [u8], args: Vec<&'a [u8]>) -> Self {
        let ret = Command::command_args_internal(command, args);
        ret.unwrap_or_else(|(command, args)| Command::Unknown { command, args })
    }

    fn command_args_internal(command: &'a [u8], mut args: Vec<&'a [u8]>) -> Result<Self, (&'a [u8], Vec<&'a [u8]>)> {
        Ok(match command {
            b"PRIVMSG" if args.len() == 2 => {
                let message = args.pop().unwrap();
                let channel = args.pop().unwrap();
                Command::Privmsg { channel, message }
            },
            _ => return Err((command, args)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_command_args() {
        let command = b"PRIVMSG";
        let args = vec![b"#foo".as_ref(), b"Hello, world!".as_ref()];
        let result = Command::with_command_args(command, args);
        let expected = Command::Privmsg {
            channel: b"#foo".as_ref(),
            message: b"Hello, world!".as_ref(),
        };
        assert_eq!(result, expected);

        let command = b"PRIVMSG";
        let args = vec![b"#foo".as_ref(), b"a".as_ref(), b"Hello, world!".as_ref()];
        let result = Command::with_command_args(command, args);
        let expected = Command::Unknown {
            command: b"PRIVMSG".as_ref(),
            args: vec![b"#foo".as_ref(), b"a".as_ref(), b"Hello, world!".as_ref()],
        };
        assert_eq!(result, expected);

        let command = b"FOO";
        let args = vec![b"BAR".as_ref(), b"baz quux".as_ref()];
        let result = Command::with_command_args(command, args);
        let expected = Command::Unknown {
            command: b"FOO".as_ref(),
            args: vec![b"BAR".as_ref(), b"baz quux".as_ref()],
        };
        assert_eq!(result, expected);
    }
}
