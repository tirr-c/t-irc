use tokio_codec::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use ircparse::Message;

#[derive(Debug)]
pub enum StreamMessage {
    Message(Message<'static>),
    Invalid(Vec<u8>),
}

impl StreamMessage {
    pub fn is_valid(&self) -> bool {
        match self {
            StreamMessage::Invalid(_) => false,
            _ => true,
        }
    }
}

pub struct IrcCodec;
impl Decoder for IrcCodec {
    type Item = StreamMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match Message::parse(src) {
            Ok((msg, left)) => {
                let msg = msg.into_owned();
                let count = src.len() - left.len();
                src.advance(count);
                Ok(Some(StreamMessage::Message(msg)))
            },
            Err(None) => {
                Ok(None)
            },
            Err(_) => {
                if let Some(newline_pos) = src.iter().position(|&b| b == b'\n') {
                    Ok(Some(
                        StreamMessage::Invalid(src.split_to(newline_pos + 1).into_iter().collect())
                    ))
                } else {
                    Ok(None)
                }
            },
        }
    }
}

impl Encoder for IrcCodec {
    type Item = Message<'static>;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.write_to(dst.writer())
    }
}
