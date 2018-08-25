use std::borrow::Cow;
use std::io::{Result as IoResult, Write};

use crate::wire;
pub use crate::command::Command;

#[derive(Debug, PartialEq, Clone)]
pub struct Tag<'a> {
    pub key: Cow<'a, [u8]>,
    pub value: Option<Cow<'a, [u8]>>,
}

impl<'a> Tag<'a> {
    pub fn into_owned(self) -> Tag<'static> {
        Tag {
            key: self.key.into_owned().into(),
            value: self.value.map(|value| value.into_owned().into()),
        }
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        writer.write_all(&self.key)?;
        if let Some(value) = &self.value {
            writer.write_all(b"=")?;
            writer.write_all(value)?;
        }
        Ok(())
    }
}

impl<'a> Into<Vec<u8>> for Tag<'a> {
    fn into(self) -> Vec<u8> {
        let mut ret = vec![];
        self.write_to(&mut ret).ok();
        ret
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Prefix<'a> {
    pub nickname: Cow<'a, [u8]>,
    pub user: Option<Cow<'a, [u8]>>,
    pub host: Option<Cow<'a, [u8]>>,
}

impl<'a> Prefix<'a> {
    pub fn into_owned(self) -> Prefix<'static> {
        Prefix {
            nickname: self.nickname.into_owned().into(),
            user: self.user.map(|user| user.into_owned().into()),
            host: self.host.map(|host| host.into_owned().into()),
        }
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        writer.write_all(b":")?;
        writer.write_all(&self.nickname)?;
        match (&self.user, &self.host) {
            (None, None) => {}
            (None, Some(host)) => {
                writer.write_all(b"@")?;
                writer.write_all(host)?;
            }
            (Some(user), Some(host)) => {
                writer.write_all(b"!")?;
                writer.write_all(user)?;
                writer.write_all(b"@")?;
                writer.write_all(host)?;
            }
            _ => unimplemented!()
        }
        Ok(())
    }
}

impl<'a> Into<Vec<u8>> for Prefix<'a> {
    fn into(self) -> Vec<u8> {
        let mut ret = vec![];
        self.write_to(&mut ret).ok();
        ret
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RawCommandAndArgs<'a> {
    pub command: Cow<'a, [u8]>,
    pub args: Vec<Cow<'a, [u8]>>,
    pub rest: Option<Cow<'a, [u8]>>,
}

impl<'a> RawCommandAndArgs<'a> {
    pub fn into_owned(self) -> RawCommandAndArgs<'static> {
        RawCommandAndArgs {
            command: self.command.into_owned().into(),
            args: self.args.into_iter().map(|arg| arg.into_owned().into()).collect(),
            rest: self.rest.map(|rest| rest.into_owned().into()),
        }
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        writer.write_all(&self.command)?;
        for arg in &self.args {
            writer.write_all(b" ")?;
            writer.write_all(arg)?;
        }
        if let Some(rest) = &self.rest {
            writer.write_all(b" :")?;
            writer.write_all(rest)?;
        }
        Ok(())
    }
}

impl<'a> Into<Vec<u8>> for RawCommandAndArgs<'a> {
    fn into(self) -> Vec<u8> {
        let mut ret = vec![];
        self.write_to(&mut ret).ok();
        ret
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message<'a> {
    pub tags: Vec<Tag<'a>>,
    pub prefix: Option<Prefix<'a>>,
    pub command: Command<'a>,
}

impl<'a> From<wire::Message<'a>> for Message<'a> {
    fn from(value: wire::Message<'a>) -> Self {
        Message {
            tags: value.tags,
            prefix: value.prefix,
            command: value.command_and_args.into(),
        }
    }
}

impl<'a> Message<'a> {
    pub fn into_owned(self) -> Message<'static> {
        Message {
            tags: self.tags.into_iter().map(|tag| tag.into_owned()).collect(),
            prefix: self.prefix.map(|prefix| prefix.into_owned()),
            command: self.command.into_owned(),
        }
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        if !self.tags.is_empty() {
            let mut first = true;
            for tag in &self.tags {
                writer.write_all(if first { b"@" } else { b";" })?;
                tag.write_to(&mut writer)?;
                first = false;
            }
            writer.write_all(b" ")?;
        }
        if let Some(prefix) = &self.prefix {
            prefix.write_to(&mut writer)?;
            writer.write_all(b" ")?;
        }
        let command: RawCommandAndArgs = self.command.clone().into();
        command.write_to(&mut writer)?;
        writer.write_all(b"\r\n")?;
        Ok(())
    }
}

impl<'a> Into<Vec<u8>> for Message<'a> {
    fn into(self) -> Vec<u8> {
        let mut ret = vec![];
        self.write_to(&mut ret).ok();
        ret
    }
}

impl<'a> Message<'a> {
    pub fn parse(data: &'a [u8]) -> Result<(Self, &'a [u8]), Option<&'a [u8]>> {
        wire::Message::parse(data).map(|(msg, left)| (From::from(msg), left))
    }
}
