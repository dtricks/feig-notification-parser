#![allow(dead_code, unused_imports)]
const CRC_PRESET: u16 = 0xFFFF;
const CRC_POLYNOM: u16 = 0x8408;

pub fn calculate_crc(data: &[u8]) -> u16 {
    let mut crc = CRC_PRESET;

    for i in 0..data.len() {
        crc ^= data[i] as u16;

        for _j in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ CRC_POLYNOM;
            } else {
                crc >>= 1
            }
        }
    }
    crc
}

pub fn check_crc(data: &[u8], expected: u16) -> bool {
    calculate_crc(data) == expected
}
