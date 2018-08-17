use lazy_static::lazy_static;
use nom::*;
use regex::bytes::Regex;

#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    key: &'a [u8],
    value: Option<&'a [u8]>,
}

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
    let key = capture.name("key").unwrap().as_bytes();
    let value = capture.name("value").map(|x| x.as_bytes());

    Ok((left, Tag { key, value }))
}

named!(tags<Vec<Tag>>, separated_nonempty_list!(char!(';'), tag));

#[derive(Debug, PartialEq)]
pub struct Prefix<'a> {
    nickname: &'a [u8],
    user: Option<&'a [u8]>,
    host: Option<&'a [u8]>,
}

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
    let nickname = capture.name("nickname").unwrap().as_bytes();
    let user = capture.name("user").map(|x| x.as_bytes());
    let host = capture.name("host").map(|x| x.as_bytes());

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
    pub command: &'a [u8],
    pub args: Vec<&'a [u8]>,
}

fn command(input: &[u8]) -> IResult<&[u8], &[u8]> {
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

    Ok((left, command.as_bytes()))
}

named!(
    message<Message>,
    do_parse!(
        tags: opt!(delimited!(char!('@'), tags, char!(' '))) >>
        prefix: opt!(delimited!(char!(':'), prefix, char!(' '))) >>
        command: command >>
        args: many0!(
            preceded!(
                char!(' '),
                alt!(
                    preceded!(char!(':'), take_until!("\r\n")) |
                    take_till!(|c: u8| match c { b' ' | b'\r' | b'\n' => true, _ => false })
                )
            )
        ) >>
        tag!("\r\n") >>
        (Message {
            tags: tags.unwrap_or_else(|| vec![]),
            prefix,
            command,
            args,
        })
    )
);

impl<'a> Message<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> Option<Self> {
        message(data).ok().and_then(|(left, ret)| if left.is_empty() { None } else { Some(ret) })
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
            command: &b"CAP"[..],
            args: vec![
                &b"LS"[..],
                &b"*"[..],
                &b"multi-prefix extended-join sasl"[..],
            ],
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
            command: &b"PRIVMSG"[..],
            args: vec![&b"#chan"[..], &b"Hey what's up!"[..]],
        };
        assert_eq!(result, Ok((left, expected)));

        let input = b"CAP REQ :sasl\r\n";
        let result = message(input);
        let left = &b""[..];
        let expected = Message {
            tags: vec![],
            prefix: None,
            command: &b"CAP"[..],
            args: vec![&b"REQ"[..], &b"sasl"[..]],
        };
        assert_eq!(result, Ok((left, expected)));
    }
}
