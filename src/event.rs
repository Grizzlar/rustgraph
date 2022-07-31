pub struct Event {
    sender: u8,
    id: u32,
    hash: u32,
    sparent: u32,
    gparent: u32,
    ts: u32
}

pub fn to_u32(bytes: &[u8]) -> u32 {
    let mut result :u32 = bytes[0] as u32;
    result += (bytes[1] as u32) << 8;
    result += (bytes[2] as u32) << 16;
    result += (bytes[3] as u32) << 24;
    result
}

impl Event {
    pub fn new(sender: u8, id: u32, hash: u32, sparent: u32, gparent: u32, ts: u32) -> Event {
        // TODO: Calculate hash instead of receiving it as input
        Event {
            sender,
            id,
            hash,
            sparent,
            gparent,
            ts
        }
    }

    pub fn from_bytes(packet: &[u8]) -> Event {
        Event {
            sender: packet[0],
            id: to_u32(&packet[1..=4]),
            hash: to_u32(&packet[5..=8]),
            sparent: to_u32(&packet[9..=12]),
            gparent: to_u32(&packet[13..=16]),
            ts: to_u32(&packet[17..=20])
        }
    }

    pub fn stringify(&self) -> String {
        format!("Event {{
            id: {}
            sender: {}
            hash: {}
            sparent: {}
            gparent: {}
            ts: {}
        }}", self.id, self.sender, self.hash, self.sparent, self.gparent, self.ts)
    }
        
    pub fn visualize(&self) {
        println!("{}", self.stringify());
    }

    pub fn to_bytes(&self) -> [u8; 21] {
        let mut bytes = [0; 21];
        bytes[0] = self.sender;
        bytes[1..=4].copy_from_slice(&self.id.to_le_bytes());
        bytes[5..=8].copy_from_slice(&self.hash.to_le_bytes());
        bytes[9..=12].copy_from_slice(&self.sparent.to_le_bytes());
        bytes[13..=16].copy_from_slice(&self.gparent.to_le_bytes());
        bytes[17..=20].copy_from_slice(&self.ts.to_le_bytes());
        bytes
    }

    pub fn sender(&self) -> u8 { self.sender }
    pub fn id(&self) -> u32 { self.id }
    pub fn hash(&self) -> u32 { self.hash }
    pub fn sparent(&self) -> u32 { self.sparent }
    pub fn gparent(&self) -> u32 { self.gparent }
    pub fn ts(&self) -> u32 { self.ts }
}