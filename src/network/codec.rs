use crate::types::{self, TryReadFrom, TryReadInto, VarInt, WriteFrom, WriteInto};
use anyhow::bail;
use bytes::{
    Buf, BytesMut,
};

use std::marker::PhantomData;

pub use crate::packets::*;

pub trait Side: Sized + 'static {}
pub struct ServerSide;
impl Side for ServerSide {}
pub struct ClientSide;
impl Side for ClientSide {}

pub trait State: Sized + 'static {
    type Inbound: Packet;
    type Outbound: Packet;
}

struct Compressor {
    compression_threshold: usize,
    compress: Compress,
    decompres: Decompress,
    deflated: BytesMut,
    inflated: BytesMut,
}

impl Compressor {
    pub fn new(deflated: BytesMut, inflated: BytesMut) -> Self {
        Self {
            compression_threshold: 256,
            compress: Compress::new(Compression::default(), false),
            decompres: Decompress::new(false),
            deflated,
            inflated,
        }
    }

    fn compress(&mut self, input: &[u8]) -> Result<&mut BytesMut, CompressError> {
        self.deflated.reserve(input.len());
        self.compress
            .compress(input, &mut self.deflated, FlushCompress::None)?;
        Ok(&mut self.deflated)
    }

    fn decompress(&mut self, input: &[u8]) -> Result<&mut BytesMut, DecompressError> {
        self.inflated.reserve(input.len());
        self.decompres
            .decompress(input, &mut self.inflated, FlushDecompress::None)?;
        Ok(&mut self.inflated)
    }
}

pub struct Encryption {
    crypter: AesCfb8,
    encrypt_index: usize,
    decrypt_index: usize,
}

pub enum DelimiterState {
    Head,
    Data(usize),
}

pub struct Delimiter {
    max_length: usize,
    state: DelimiterState,
}

impl Delimiter {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            state: DelimiterState::Head,
        }
    }

    fn read_head(&self, src: &mut BytesMut) -> anyhow::Result<Option<usize>> {
        // Try to read VarInt if unable then yield and try again later.
        match VarInt::try_read(src) {
            Ok(length) => {
                let length = *length as usize;
                if length > self.max_length {
                    bail!("Packet lenght exceeds {}.", self.max_length);
                }
                Ok(Some(length))
            }
            Err(types::Error::NotEnoughBytes) => Ok(None),
            e @ Err(_) => {
                e?;
                Ok(None)
            }
        }
    }
}

pub struct PacketCodec<S: Stage, D: Side, P: Protocol> {
    protocol: PhantomData<P>,
    side: PhantomData<D>,
    stage: PhantomData<S>,
    compressor: Option<Compressor>,
    encryption: Option<Encryption>,
    delimiter: Delimiter,
}

impl<P: Protocol> State for PacketCodec<HandshakeStage, ServerSide, P> {
    type Outbound = P::ServerBoundHandshakePacket;
    type Inbound = P::ClientBoundHandshakePacket;
}

impl<P: Protocol> State for PacketCodec<StatusStage, ServerSide, P> {
    type Outbound = P::ServerBoundStatusPacket;
    type Inbound = P::ClientBoundStatusPacket;
}

impl<P: Protocol> State for PacketCodec<LoginStage, ServerSide, P> {
    type Outbound = P::ServerBoundLoginPacket;
    type Inbound = P::ClientBoundLoginPacket;
}

impl<P: Protocol> State for PacketCodec<PlayStage, ServerSide, P> {
    type Outbound = P::ServerBoundPlayPacket;
    type Inbound = P::ClientBoundPlayPacket;
}

impl<P: Protocol> State for PacketCodec<HandshakeStage, ClientSide, P> {
    type Outbound = P::ClientBoundHandshakePacket;
    type Inbound = P::ServerBoundHandshakePacket;
}

impl<P: Protocol> State for PacketCodec<StatusStage, ClientSide, P> {
    type Outbound = P::ClientBoundStatusPacket;
    type Inbound = P::ServerBoundStatusPacket;
}

impl<P: Protocol> State for PacketCodec<LoginStage, ClientSide, P> {
    type Outbound = P::ClientBoundLoginPacket;
    type Inbound = P::ServerBoundLoginPacket;
}

impl<P: Protocol> State for PacketCodec<PlayStage, ClientSide, P> {
    type Outbound = P::ClientBoundPlayPacket;
    type Inbound = P::ServerBoundPlayPacket;
}

impl<S, D, P> PacketCodec<S, D, P>
where
    S: Stage,
    D: Side,
    P: Protocol,
{
    pub fn new(delimiter: Delimiter) -> Self {
        PacketCodec {
            protocol: Default::default(),
            side: Default::default(),
            stage: Default::default(),
            compressor: None,
            encryption: None,
            delimiter,
        }
    }
}

macro_rules! state_change {
    {$($from:ident => [$($to:ident),*]),*} => {
        $(
            $(
            impl<D, P> From<PacketCodec<$from, D, P>> for PacketCodec<$to, D, P>
            where
                D: Side,
                P: Protocol,
            {
                fn from(from: PacketCodec<$from, D, P>) -> Self {
                    PacketCodec {
                        protocol: from.protocol,
                        side: from.side,
                        stage: Default::default(),
                        compressor: from.compressor,
                        encryption: from.encryption,
                        delimiter: from.delimiter,
                    }
                }
            }
            )*
        )*
    };
}

state_change! {
    HandshakeStage => [StatusStage, LoginStage],
    LoginStage => [PlayStage]
}

impl<S, D, P> Decoder for PacketCodec<S, D, P>
where
    S: Stage,
    D: Side,
    P: Protocol,
    Self: State,
{
    type Item = <Self as State>::Inbound;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decrypt if encryption is enabled, keeping track of what have been decrypted so far.
        if let Some(encryption) = self.encryption.as_mut() {
            encryption.crypter.decrypt(&mut src[encryption.encrypt_index..]);
            encryption.encrypt_index = src.len();
        }

        // Track the state of the codec are we reading data or the head.
        let length = match self.delimiter.state {
            DelimiterState::Head => match self.delimiter.read_head(src)? {
                Some(n) => {
                    self.delimiter.state = DelimiterState::Data(n);
                    n
                }
                None => return Ok(None),
            },
            DelimiterState::Data(n) => n,
        };

        // Wait for the remaning data
        if src.remaining() < length {
            return Ok(None);
        }

        // Reset the state
        self.delimiter.state = DelimiterState::Head;

        // Decress the decrypt index, since we are have removed data from the buffer.
        if let Some(encryption) = self.encryption.as_mut() {
            encryption.decrypt_index -= length;
        }

        // We should only operate at data upto lenght
        let mut src = src.split_to(length);

        // Deflate if compression is enabled.
        let mut src = if let Some(compressor) = self.compressor.as_mut() {
            let data_length: VarInt = src.try_read()?;
            let data_length = *data_length as usize;

            // Do not exceed the internal buffer
            if data_length > compressor.inflated.capacity() {
                bail!("Inflated buffer capacity exceeded");
            }

            if data_length > 0 {
                // Split the data of from the internal buffer
                compressor.decompress(&src)?.split()
            } else {
                // The data is not compressed
                src
            }
        } else {
            // Compression is not enabled
            src
        };

        // Try to read the packet from the buffer.
        Ok(Some(src.try_read()?))
    }
}

impl<P, D, S> Encoder<<Self as State>::Outbound> for PacketCodec<S, D, P>
where
    S: Stage,
    D: Side,
    P: Protocol,
    Self: State,
{
    type Error = anyhow::Error;
    fn encode(
        &mut self,
        item: <Self as State>::Outbound,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let mut buf = [0u8; 10];

        dst.extend(&buf[..]);

        let packet_size = dst.write(&item);

        let lenght_size = (&VarInt::from(packet_size as i32)).write(&mut &mut buf[..5]);
        let lenght_offset = 10 - lenght_size;

        dst[(lenght_offset)..10].copy_from_slice(&buf[..lenght_size]);

        let dst = if let Some(compressor) = self.compressor.as_mut() {
            if packet_size > compressor.compression_threshold {
                compressor.compress(&dst)?
            } else {
                (&VarInt::from(0)).write(&mut &mut buf[5..10]);
                dst
            }
        } else {
            dst
        };

        dst.advance(lenght_offset);

        if let Some(encryption) = self.encryption.as_mut() {
            encryption.crypter.encrypt(dst);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{HandshakeStage, LoginStage, PacketCodec, PlayStage, ServerSide, Delimiter};
    // use crate::v1_15_2::Protocol;

    #[test]
    fn test() {
        // let codec: PacketCodec<HandshakeStage, ServerSide, Protocol> = PacketCodec::new(Delimiter::new(50_000));
        // let codec: PacketCodec<LoginStage, _, _> = codec.into();
        // let _codec: PacketCodec<PlayStage, _, _> = codec.into();
    }
}
