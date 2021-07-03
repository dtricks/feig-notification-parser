#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub enum FeigMessage {
    Data {
        raw: Vec<u8>,
        status: u8,
        data: Vec<TagRead>,
        command_code: u8,
        message_code: u8,
        com_adr: u8,
        crc: u16,
        len: u16,
    },
    Keepalive {
        raw: Vec<u8>,
        crc: u16,
        command_code: u8,
        com_adr: u8,
        status: u8,
        flags_a: u8,
        flags_b: u8,
        message_code: u8,
        len: u16,
    },
    Generic(Vec<u8>),
}

#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub struct TagRead {
    pub record_len: u16,
    pub transponder_type: u8,
    pub idd_t: u8,
    pub idd_len: u8,
    pub serial_number: Vec<u8>,
    pub time: u32,
    pub mac: Vec<u8>,
}
