pub mod player_info_packet {
    pub struct Packet {
        players: Vec<Player>,
    }
    pub struct Player {
        uuid: uuid::Uuid,
        action: Action,
    }
    pub enum Action {
        AddPlayer(AddPlayer),
        UpdateGamemode(UpdateGamemode),
        UpdateLatency(UpdateLatency),
        UpdateDisplayName(UpdateDisplayName),
        RemovePlayer,
    }
    pub struct AddPlayer {
        name: String,
        properties: Vec<Property>,
        gamemode: VarInt,
        ping: VarInt,
        display_name: Option<String>,
    }
    pub struct Property {
        name: String,
        value: String,
        is_signed: bool,
        signature: Option<String>,
    }
    pub struct UpdateGamemode {
        gamemode: VarInt,
    }
    pub struct UpdateLatency {
        ping: VarInt,
    }
    pub struct UpdateDisplayName {
        display_name: Option<String>,
    }
    impl feather_protocol::Packet for Packet {
        fn id(&self) -> u64 {
            0
        }
        fn name(&self) -> &str {
            "player_info"
        }
    }
}
