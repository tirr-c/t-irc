use std::borrow::Cow;
use crate::data::RawCommandAndArgs;

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'a> {
    Privmsg {
        channel: Cow<'a, [u8]>,
        message: Cow<'a, [u8]>,
    },
    Unknown {
        command: Cow<'a, [u8]>,
        args: Vec<Cow<'a, [u8]>>,
        rest: Option<Cow<'a, [u8]>>,
    },
}

impl<'a> From<RawCommandAndArgs<'a>> for Command<'a> {
    fn from(RawCommandAndArgs { command, mut args, rest }: RawCommandAndArgs<'a>) -> Self {
        match (command.as_ref(), rest, args.len()) {
            (b"PRIVMSG", Some(rest), 1) => {
                let channel = args.pop().unwrap();
                let message = rest;
                Command::Privmsg { channel, message }
            },
            (_, rest, _) => Command::Unknown { command, args, rest },
        }
    }
}

impl<'a> Into<RawCommandAndArgs<'a>> for Command<'a> {
    fn into(self) -> RawCommandAndArgs<'a> {
        let (command, args, rest) = match self {
            Command::Privmsg { channel, message } => {
                (b"PRIVMSG".as_ref().into(), vec![channel], Some(message))
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

impl<'a> Command<'a> {
    pub fn into_owned(self) -> Command<'static> {
        use self::Command::*;

        match self {
            Privmsg { channel, message } => Privmsg {
                channel: channel.into_owned().into(),
                message: message.into_owned().into(),
            },
            Unknown { command, args, rest } => Unknown {
                command: command.into_owned().into(),
                args: args.into_iter().map(|arg| arg.into_owned().into()).collect(),
                rest: rest.map(|rest| rest.into_owned().into()),
            },
        }
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
