#![feature(rust_2018_preview)]

pub mod wire;
pub mod command;

pub struct Message<'a> {
    pub tags: Vec<wire::Tag<'a>>,
    pub prefix: Option<wire::Prefix<'a>>,
    pub command: command::Command<'a>,
}

impl<'a> From<wire::Message<'a>> for Message<'a> {
    fn from(value: wire::Message<'a>) -> Self {
        Message {
            tags: value.tags,
            prefix: value.prefix,
            command: command::Command::with_command_args(value.command, value.args),
        }
    }
}

impl<'a> Message<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        wire::Message::parse(data).map(From::from)
    }
}
