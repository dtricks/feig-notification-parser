use serde::Serialize;

#[derive(Debug, Eq, PartialEq, PartialOrd, Serialize)]
pub enum FeigMessage {
    Data {
        raw: Vec<u8>,
        status: u8,
        tags: Vec<TagRead>,
        command_code: u8,
        message_code: u8,
        com_adr: u8,
        crc: u16,
        len: u16,
        correct_crc: bool,
    },
    Keepalive {
        raw: Vec<u8>,
        crc: u16,
        command_code: u8,
        com_adr: u8,
        status: u8,
        flags_a: u8,
        flags_b: u8,
        flag_temp_alarm: bool,
        flag_false_power: bool,
        flag_wrong_antenna_impedance: bool,
        flag_dc_power_error: bool,
        flag_noise: bool,
        message_code: u8,
        len: u16,
        correct_crc: bool,
    },
    Generic(Vec<u8>),
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Serialize)]
pub struct TagRead {
    pub record_len: u16,
    pub transponder_type: TransponderType,
    pub idd_t: IDDT,
    pub idd_len: u8,
    pub serial_number: Vec<u8>,
    pub time: u32,
    pub mac: String,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Serialize)]
pub enum TransponderType {
    Unknown(u8),
    ICode1,
    Iso15693Tag,
    Iso18000_3M3,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Serialize)]
pub enum IDDT {
    Unknown(u8),
    EPC,
    UID,
}
