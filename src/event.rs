pub struct Event {
    sender: u8,
    sparent: u32,
    gparent: u32,
    ts: u32
}

fn to_u32(bytes: &[u8]) -> u32 {
    let mut result :u32 = bytes[0] as u32;
    result += (bytes[1] as u32) << 8;
    result += (bytes[2] as u32) << 16;
    result += (bytes[3] as u32) << 24;
    result
}

impl Event {
    pub fn new(packet: &[u8]) -> Event {
        Event {
            sender: packet[0],
            sparent: to_u32(&packet[1..=4]),
            gparent: to_u32(&packet[5..=8]),
            ts: to_u32(&packet[9..=12])
        }
    }

    pub fn visualize(&self) {
        println!("Event {{
    sender: {}
    sparent: {}
    gparent: {}
    ts: {}
}}", self.sender, self.sparent, self.gparent, self.ts);
    }

    pub fn to_bytes(&self) -> [u8; 13] {
        let mut bytes = [0; 13];
        bytes[0] = self.sender;
        bytes[1..=4].copy_from_slice(&self.sparent.to_le_bytes());
        bytes[5..=8].copy_from_slice(&self.gparent.to_le_bytes());
        bytes[9..=12].copy_from_slice(&self.ts.to_le_bytes());
        bytes
    }
}