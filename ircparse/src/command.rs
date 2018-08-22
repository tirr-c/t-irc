use crate::data::RawCommandAndArgs;

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'a> {
    Privmsg {
        channel: &'a [u8],
        message: &'a [u8],
    },
    Unknown {
        command: &'a [u8],
        args: Vec<&'a [u8]>,
        rest: Option<&'a [u8]>,
    },
}

impl<'a> From<RawCommandAndArgs<'a>> for Command<'a> {
    fn from(RawCommandAndArgs { command, mut args, rest }: RawCommandAndArgs<'a>) -> Self {
        match (command, rest) {
            (b"PRIVMSG", Some(rest)) if args.len() == 1 => {
                let channel = args.pop().unwrap();
                let message = rest;
                Command::Privmsg { channel, message }
            },
            _ => Command::Unknown { command, args, rest },
        }
    }
}

impl<'r, 'a> Into<RawCommandAndArgs<'a>> for &'r Command<'a> {
    fn into(self) -> RawCommandAndArgs<'a> {
        let mut args = vec![];
        let (command, rest) = match self {
            &Command::Privmsg { channel, message } => {
                args.push(channel);
                (b"PRIVMSG".as_ref(), Some(message))
            },
            &Command::Unknown { command, ref args, rest } => {
                return RawCommandAndArgs { command, args: args.clone(), rest };
            },
        };
        RawCommandAndArgs { command, args, rest }
    }
}

impl<'a> Into<RawCommandAndArgs<'a>> for Command<'a> {
    fn into(self) -> RawCommandAndArgs<'a> {
        let (command, args, rest) = match self {
            Command::Privmsg { channel, message } => {
                (b"PRIVMSG".as_ref(), vec![channel], Some(message))
            },
            Command::Unknown { command, args, rest } => {
                (command, args, rest)
            },
        };
        RawCommandAndArgs { command, args, rest }
    }
}

impl<'a> Into<Vec<u8>> for Command<'a> {
    fn into(self) -> Vec<u8> {
        <Command as Into<RawCommandAndArgs>>::into(self).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_command_args() {
        let input = RawCommandAndArgs {
            command: b"PRIVMSG".as_ref(),
            args: vec![b"#foo".as_ref()],
            rest: Some(b"Hello, world!".as_ref()),
        };
        let result = Command::from(input);
        let expected = Command::Privmsg {
            channel: b"#foo".as_ref(),
            message: b"Hello, world!".as_ref(),
        };
        assert_eq!(result, expected);

        let input = RawCommandAndArgs {
            command: b"PRIVMSG".as_ref(),
            args: vec![b"#foo".as_ref(), b"a".as_ref()],
            rest: Some(b"Hello, world!".as_ref()),
        };
        let result = Command::from(input);
        let expected = Command::Unknown {
            command: b"PRIVMSG".as_ref(),
            args: vec![b"#foo".as_ref(), b"a".as_ref()],
            rest: Some(b"Hello, world!".as_ref()),
        };
        assert_eq!(result, expected);

        let input = RawCommandAndArgs {
            command: b"FOO".as_ref(),
            args: vec![b"BAR".as_ref()],
            rest: Some(b"baz quux".as_ref()),
        };
        let result = Command::from(input);
        let expected = Command::Unknown {
            command: b"FOO".as_ref(),
            args: vec![b"BAR".as_ref()],
            rest: Some(b"baz quux".as_ref()),
        };
        assert_eq!(result, expected);
    }
}
