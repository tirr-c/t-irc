use std::borrow::Cow;
use lazy_static::lazy_static;
use nom::*;
use regex::bytes::Regex;
use crate::data::{RawCommandAndArgs, Tag, Prefix};

fn tag(input: &[u8]) -> IResult<&[u8], Tag> {
    lazy_static! {
        static ref TAG_REGEX: Regex = Regex::new(
            r"^(?P<key>(?:[A-Za-z0-9\-.]+/)?[A-Za-z0-9\-]+)(?:=(?P<value>[^\x00\r\n; ]*))?"
        ).unwrap();
    }

    let capture = TAG_REGEX.captures(input);
    let capture = match capture {
        None => {
            return Err(nom::Err::Error(error_position!(
                input,
                nom::ErrorKind::Custom(0)
            )))
        }
        Some(x) => x,
    };
    let full_match = capture.get(0).unwrap();
    let left = &input[full_match.end()..];
    let key = capture.name("key").unwrap().as_bytes().into();
    let value = capture.name("value").map(|x| x.as_bytes().into());

    Ok((left, Tag { key, value }))
}

named!(tags<Vec<Tag>>, separated_nonempty_list!(char!(';'), tag));

fn prefix(input: &[u8]) -> IResult<&[u8], Prefix> {
    lazy_static! {
        static ref PREFIX_REGEX: Regex =
            Regex::new(r"^(?P<nickname>[^!@ ]+)(?:(?:!(?P<user>[^@ ]+))?@(?P<host>[^ ]+))?")
                .unwrap();
    }

    let capture = PREFIX_REGEX.captures(input);
    let capture = match capture {
        None => {
            return Err(nom::Err::Error(error_position!(
                input,
                nom::ErrorKind::Custom(1)
            )))
        }
        Some(x) => x,
    };
    let full_match = capture.get(0).unwrap();
    let left = &input[full_match.end()..];
    let nickname = capture.name("nickname").unwrap().as_bytes().into();
    let user = capture.name("user").map(|x| x.as_bytes().into());
    let host = capture.name("host").map(|x| x.as_bytes().into());

    Ok((
        left,
        Prefix {
            nickname,
            user,
            host,
        },
    ))
}

#[derive(Debug, PartialEq)]
pub(crate) struct Message<'a> {
    pub tags: Vec<Tag<'a>>,
    pub prefix: Option<Prefix<'a>>,
    pub command_and_args: RawCommandAndArgs<'a>,
}

fn command(input: &[u8]) -> IResult<&[u8], Cow<[u8]>> {
    lazy_static! {
        static ref COMMAND_REGEX: Regex = Regex::new(r"^[A-Za-z_\-]+|[0-9]{3}").unwrap();
    }

    let command = COMMAND_REGEX.find(input);
    let command = match command {
        None => {
            return Err(nom::Err::Error(error_position!(
                input,
                nom::ErrorKind::Custom(2)
            )))
        }
        Some(x) => x,
    };
    let left = &input[command.end()..];

    Ok((left, command.as_bytes().into()))
}

named!(
    message<Message>,
    do_parse!(
        tags: opt!(delimited!(char!('@'), tags, char!(' '))) >>
        prefix: opt!(delimited!(char!(':'), prefix, char!(' '))) >>
        command: command >>
        args: many_till!(
            preceded!(
                char!(' '),
                map!(
                    take_till!(|c: u8| match c { b' ' | b'\r' | b'\n' => true, _ => false }),
                    Into::into
                )
            ),
            peek!(alt!(tag!(" :") | tag!("\r\n")))
        ) >>
        rest: opt!(
            preceded!(
                tag!(" :"),
                take_until!("\r\n")
            )
        ) >>
        tag!("\r\n") >>
        (Message {
            tags: tags.unwrap_or_else(|| vec![]),
            prefix,
            command_and_args: RawCommandAndArgs {
                command,
                args: args.0,
                rest: rest.map(Into::into),
            },
        })
    )
);

impl<'a> Message<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Result<(Self, &'a [u8]), Option<&'a [u8]>> {
        message(data)
            .map(|(left, msg)| (msg, left))
            .map_err(|err| match err {
                | nom::Err::Incomplete(_)
                => None,
                | nom::Err::Error(nom::Context::Code(input, _))
                | nom::Err::Failure(nom::Context::Code(input, _))
                => Some(input),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag() {
        let input = b"netsplit=tur,ty";
        let result = tag(input);
        let left = &b""[..];
        let key = &b"netsplit"[..];
        let value = Some(&b"tur,ty"[..]);
        assert_eq!(result, Ok((left, Tag { key, value })));

        let input = b"rose";
        let result = tag(input);
        let left = &b""[..];
        let key = &b"rose"[..];
        let value = None;
        assert_eq!(result, Ok((left, Tag { key, value })));

        let input = b"id=123AB;";
        let result = tag(input);
        let left = &b";"[..];
        let key = &b"id"[..];
        let value = Some(&b"123AB"[..]);
        assert_eq!(result, Ok((left, Tag { key, value })));
    }

    #[test]
    fn test_tags() {
        let input = b"url=;rose;netsplit=tur,ty ";
        let result = tags(input);
        let result = result.as_ref().map(|(a, b)| (*a, b.as_slice()));
        let left = &b" "[..];
        let expected = &[
            Tag {
                key: &b"url"[..],
                value: Some(&b""[..]),
            },
            Tag {
                key: &b"rose"[..],
                value: None,
            },
            Tag {
                key: &b"netsplit"[..],
                value: Some(&b"tur,ty"[..]),
            },
        ][..];
        assert_eq!(result, Ok((left, expected)));
    }

    #[test]
    fn test_prefix() {
        let input = b"dan!d@localhost ";
        let result = prefix(input);
        let left = &b" "[..];
        let expected = Prefix {
            nickname: &b"dan"[..],
            user: Some(&b"d"[..]),
            host: Some(&b"localhost"[..]),
        };
        assert_eq!(result, Ok((left, expected)));

        let input = b"localhost ";
        let result = prefix(input);
        let left = &b" "[..];
        let expected = Prefix {
            nickname: &b"localhost"[..],
            user: None,
            host: None,
        };
        assert_eq!(result, Ok((left, expected)));

        let input = b"dan@localhost ";
        let result = prefix(input);
        let left = &b" "[..];
        let expected = Prefix {
            nickname: &b"dan"[..],
            user: None,
            host: Some(&b"localhost"[..]),
        };
        assert_eq!(result, Ok((left, expected)));
    }

    #[test]
    fn test_raw_message() {
        let input = b":irc.example.com CAP LS * :multi-prefix extended-join sasl\r\n";
        let result = message(input);
        let left = &b""[..];
        let expected = Message {
            tags: vec![],
            prefix: Some(Prefix {
                nickname: &b"irc.example.com"[..],
                user: None,
                host: None,
            }),
            command_and_args: RawCommandAndArgs {
                command: &b"CAP"[..],
                args: vec![
                    &b"LS"[..],
                    &b"*"[..],
                ],
                rest: Some(&b"multi-prefix extended-join sasl"[..]),
            }
        };
        assert_eq!(result, Ok((left, expected)));

        let input = b"@id=234AB :dan!d@localhost PRIVMSG #chan :Hey what's up!\r\n";
        let result = message(input);
        let left = &b""[..];
        let expected = Message {
            tags: vec![Tag {
                key: &b"id"[..],
                value: Some(&b"234AB"[..]),
            }],
            prefix: Some(Prefix {
                nickname: &b"dan"[..],
                user: Some(&b"d"[..]),
                host: Some(&b"localhost"[..]),
            }),
            command_and_args: RawCommandAndArgs {
                command: &b"PRIVMSG"[..],
                args: vec![&b"#chan"[..]],
                rest: Some(&b"Hey what's up!"[..]),
            },
        };
        assert_eq!(result, Ok((left, expected)));

        let input = b"CAP REQ :sasl\r\n";
        let result = message(input);
        let left = &b""[..];
        let expected = Message {
            tags: vec![],
            prefix: None,
            command_and_args: RawCommandAndArgs {
                command: &b"CAP"[..],
                args: vec![&b"REQ"[..]],
                rest: Some(&b"sasl"[..]),
            },
        };
        assert_eq!(result, Ok((left, expected)));
    }
}
