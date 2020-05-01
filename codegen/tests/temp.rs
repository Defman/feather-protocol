mod protocol {
    pub mod server_bound {
        pub mod handshaking {
            pub enum Packet {}
        }
        pub mod status {
            pub enum Packet {}
        }
        pub mod login {
            pub enum Packet {}
        }
        pub mod play {
            pub enum Packet {}
        }
    }
    pub mod client_bound {
        pub mod handshaking {
            pub enum Packet {
                Ma(Ma),
            }
            impl From<Ma> for Packet {
                fn from(packet: Ma) -> Self {
                    Packet::Ma(packet)
                }
            }
            pub use ma::Ma;
            mod ma {
                pub struct Ma {
                    username: String,
                }
            }
        }
        pub mod status {
            pub enum Packet {}
        }
        pub mod login {
            pub enum Packet {}
        }
        pub mod play {
            pub enum Packet {}
        }
    }
}
mod shared {}
