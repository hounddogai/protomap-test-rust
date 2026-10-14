//! Shared configuration and raw protobuf support for the operations binaries.

use acme_proto::acme::common::v1::{Actor, RequestContext};
use prost::bytes::{Buf, BufMut};
use tonic::{
    Status,
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
};

pub fn context() -> anyhow::Result<RequestContext> {
    Ok(RequestContext {
        tenant_id: std::env::var("ACME_TENANT")?,
        actor: Some(Actor {
            user_id: "platform-operations".into(),
            service_account: true,
            ..Default::default()
        }),
        ..Default::default()
    })
}

pub fn channel() -> anyhow::Result<tonic::transport::Channel> {
    let address = std::env::var("ACME_MESH").unwrap_or_else(|_| "http://127.0.0.1:15001".into());
    Ok(tonic::transport::Channel::from_shared(address)?
        .timeout(std::time::Duration::from_secs(30))
        .connect_lazy())
}

/// The operator supplies a protobuf wire payload and a method path for APIs the named commands do not cover.
#[derive(Default)]
pub struct RawCodec;
pub struct RawEncoder;
pub struct RawDecoder;

impl Codec for RawCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;
    type Encoder = RawEncoder;
    type Decoder = RawDecoder;
    fn encoder(&mut self) -> Self::Encoder {
        RawEncoder
    }
    fn decoder(&mut self) -> Self::Decoder {
        RawDecoder
    }
}

impl Encoder for RawEncoder {
    type Item = Vec<u8>;
    type Error = Status;
    fn encode(&mut self, item: Self::Item, output: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        output.put_slice(&item);
        Ok(())
    }
}

impl Decoder for RawDecoder {
    type Item = Vec<u8>;
    type Error = Status;
    fn decode(&mut self, input: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let remaining = input.remaining();
        Ok(Some(input.copy_to_bytes(remaining).to_vec()))
    }
}
