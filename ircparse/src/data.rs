use std::io::{Result as IoResult, Write};

use crate::wire;
pub use crate::command::Command;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Tag<'a> {
    pub key: &'a [u8],
    pub value: Option<&'a [u8]>,
}

impl<'a> Tag<'a> {
    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        writer.write_all(self.key)?;
        if let Some(value) = self.value {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Prefix<'a> {
    pub nickname: &'a [u8],
    pub user: Option<&'a [u8]>,
    pub host: Option<&'a [u8]>,
}

impl<'a> Prefix<'a> {
    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        writer.write_all(b":")?;
        writer.write_all(self.nickname)?;
        match (self.user, self.host) {
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
    pub command: &'a [u8],
    pub args: Vec<&'a [u8]>,
    pub rest: Option<&'a [u8]>,
}

impl<'a> RawCommandAndArgs<'a> {
    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        writer.write_all(self.command)?;
        for &arg in &self.args {
            writer.write_all(b" ")?;
            writer.write_all(arg)?;
        }
        if let Some(rest) = self.rest {
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
    pub fn write_to<W: Write>(&self, mut writer: W) -> IoResult<()> {
        if !self.tags.is_empty() {
            let mut first = true;
            for &tag in &self.tags {
                writer.write_all(if first { b"@" } else { b";" })?;
                tag.write_to(&mut writer)?;
                first = false;
            }
            writer.write_all(b" ")?;
        }
        if let Some(prefix) = self.prefix {
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
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        wire::Message::parse(data).map(From::from)
    }
}
