use std::marker::PhantomData;
use bytes::{Buf, BufMut};
use crate::packets::{direction, stage, Protocol};

pub trait Encoder<P, D, S>
where
    P: Protocol,
    D: direction::Direction,
    S: stage::Stage,
{
    pub fn encode(packet: impl Packet<D, S>) -> impl BufMut;
}

pub trait Decoder<P, D, S>
where
    P: Protocol,
    D: direction::Direction,
    S: stage::Stage,
{
    pub fn decode(buf: impl Buf) -> impl Packet;
}

struct FeatherCodec<P: Protocol>;

macro_rules! encoder {
    ($direction:path, $stage:path) => {
        impl<P: Protocol> Encoder<P, $direction, $stage> for FeatherCodec<P> {
            fn encode(packet: impl Packet<D, S>) -> impl BufMut {
                unimplemented!()
            }
        }        
    };
}

encoder!(direction::Client, stage::Handshaking);
encoder!(direction::Client, stage::Status);
encoder!(direction::Client, stage::Login);
encoder!(direction::Client, stage::Play);

encoder!(direction::Server, stage::Handshaking);
encoder!(direction::Server, stage::Status);
encoder!(direction::Server, stage::Login);
encoder!(direction::Server, stage::Play);

macro_rules! decoder {
    ($direction:path, $stage:path) => {
        impl<P: Protocol> Decoder<P, $direction, $stage> for FeatherCodec<P> {
            fn encode(packet: impl Packet<D, S>) -> impl BufMut {
                unimplemented!()
            }
        }        
    };
}


decoder!(direction::Client, stage::Handshaking);
decoder!(direction::Client, stage::Status);
decoder!(direction::Client, stage::Login);
decoder!(direction::Client, stage::Play);

decoder!(direction::Server, stage::Handshaking);
decoder!(direction::Server, stage::Status);
decoder!(direction::Server, stage::Login);
decoder!(direction::Server, stage::Play);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoder_test() {
        // type codec = FeatherCodec<Protocol>;
        // let packet = ...;
        // let packet_encoded = codec::encode::<Client, Play>(packet);
        // let packet_decoded = codec::decode::<Client, Play>(packet_encoded);

        // assert_eq!(packet_decoded, packet);
    }
}
