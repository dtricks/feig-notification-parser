#![allow(dead_code)]

use nom::branch::alt;
use nom::bytes::complete::take;
//use nom::complete::take;
use nom::IResult;
//use nom::error::Error;
use nom::number::complete::{be_u16, le_u16};
use nom::number::complete::{be_u32, le_u8};
use nom::Err::Error;
//use nom::IResult;

use crate::crc::{calculate_crc, check_crc};
use crate::feig_types::{FeigMessage, TransponderType};
use crate::feig_types::{TagRead, IDDT};

const MASK_TEMP_ALARM: u8 = 0b1000_0000;
const MASK_FALSE_POWER: u8 = 0b0001_0000;
const MASK_WRONG_ANTENNA_IMPEDANCE: u8 = 0b0000_0100;
const MASK_NOISE: u8 = 0b0000_0010;
const MASK_DC_POWER_ERROR: u8 = 0b0000_0100;

pub fn parse_keepalive(input: &[u8]) -> IResult<&[u8], FeigMessage> {
    let raw = input.clone();
    let (input, message_code) = check_message_code(input)?;
    let (input, len) = get_message_length(input)?;
    let (input, com_adr) = get_com_adr(input)?;
    let (input, command_code) = get_command_code(input)?;
    if command_code[0] != 0x6e {
        return Err(Error(nom::error::Error::new(
            command_code,
            nom::error::ErrorKind::Not,
        )));
    }
    let (input, status) = get_status(input)?;
    let (input, keepalive_data) = get_keepalive_data(input)?;
    let flag_temp_alarm = parse_bit_pattern(&keepalive_data[0], &MASK_TEMP_ALARM);
    let flag_false_power = parse_bit_pattern(&keepalive_data[0], &MASK_FALSE_POWER);
    let flag_wrong_antenna_impedance =
        parse_bit_pattern(&keepalive_data[0], &MASK_WRONG_ANTENNA_IMPEDANCE);
    let flag_noise = parse_bit_pattern(&keepalive_data[0], &MASK_NOISE);
    let flag_dc_power_error = parse_bit_pattern(&keepalive_data[1], &MASK_DC_POWER_ERROR);

    let (input, crc) = get_crc(input)?;
    let correct_crc = check_crc(&raw[..(raw.len() - 2)], crc);
    Ok((
        input,
        FeigMessage::Keepalive {
            raw: raw.into(),
            status: status[0],
            flags_a: keepalive_data[0],
            com_adr: com_adr[0],
            command_code: command_code[0],
            flags_b: keepalive_data[1],
            crc,
            len,
            message_code: message_code[0],
            flag_temp_alarm,
            flag_false_power,
            flag_wrong_antenna_impedance,
            flag_dc_power_error,
            flag_noise,
            correct_crc,
        },
    ))
}

pub fn check_message_code(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(1u8)(input)
}

pub fn get_message_length(input: &[u8]) -> IResult<&[u8], u16> {
    be_u16(input)
}

pub fn get_com_adr(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(1u8)(input)
}

pub fn get_command_code(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(1u8)(input)
}

pub fn get_status(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(1u8)(input)
}

pub fn get_keepalive_data(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(2u8)(input)
}

pub fn get_crc(input: &[u8]) -> IResult<&[u8], u16> {
    le_u16(input)
}

pub fn get_crc_data(input: &[u8], len: u16) -> IResult<&[u8], &[u8]> {
    take(len)(input)
}

pub fn parse_data_message(input: &[u8]) -> IResult<&[u8], FeigMessage> {
    // 02 0029 00 22 00 a1 02 0001 001d84000e3400000874000000000000001328141d27ea001c9b0709572bf3
    // mc    l ca cc st    nf  cnt
    let raw = input.clone();
    let (input, message_code) = check_message_code(input)?;
    let (input, len) = get_message_length(input)?;
    let (input, com_adr) = get_com_adr(input)?;
    let (input, command_code) = get_command_code(input)?;
    if command_code[0] != 0x22 {
        return Err(Error(nom::error::Error::new(
            command_code,
            nom::error::ErrorKind::Not,
        )));
    }
    let (input, status) = get_status(input)?;
    let (input, _a1) = le_u8(input)?;
    let (input, _nf) = le_u8(input)?;
    let (input, count) = get_tag_count(input)?;
    let mut tags = vec![];
    let mut input = input;
    for _ in 0..count {
        let (input_new, tr) = parse_tag_read(input)?;
        input = input_new;
        tags.push(tr);
    }
    let (input, crc) = get_crc(input)?;
    let correct_crc = check_crc(&raw[..(raw.len() - 2)], crc);
    Ok((
        input,
        FeigMessage::Data {
            raw: raw.into(),
            status: status[0],
            tags,
            command_code: command_code[0],
            message_code: message_code[0],
            com_adr: com_adr[0],
            crc,
            len,
            correct_crc,
        },
    ))
}

pub fn parse_tag_read(input: &[u8]) -> IResult<&[u8], TagRead> {
    // 02 00 01 | 001d 84 00 0e 3400000874000000000000001328 14207e9d 001c9b070957 8ab2
    // nf st cc | coun tt it len                         idd     time          mac  crc
    //count u16 0001
    let (input, record_len) = get_message_length(input)?;
    //transponder type
    let (input, ttu8) = le_u8(input)?;
    let tt = match ttu8 {
        0x01u8 => TransponderType::ICode1,
        0x03u8 => TransponderType::Iso15693Tag,
        0x84u8 => TransponderType::Iso18000_3M3,
        x => TransponderType::Unknown(x),
    };
    //idd type
    let (input, itu8) = le_u8(input)?;
    let it = match itu8 {
        0x00u8 => IDDT::EPC,
        0x02u8 => IDDT::UID,
        x => IDDT::Unknown(x),
    };
    //idd len
    let (input, idd_len) = le_u8(input)?;
    //idd
    let (input, idd) = take(idd_len)(input)?;
    //time
    let (input, time) = be_u32(input)?;
    //mac-addr
    let (input, mac) = take(6u8)(input)?;
    let mac = format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    Ok((
        input,
        TagRead {
            record_len,
            transponder_type: tt,
            idd_t: it,
            idd_len,
            serial_number: idd.into(),
            time,
            mac: mac.into(),
        },
    ))
}

pub fn parse_bit_pattern(input: &u8, bit_pattern: &u8) -> bool {
    input & bit_pattern == *bit_pattern
}

pub fn get_tag_count(input: &[u8]) -> IResult<&[u8], u16> {
    be_u16(input)
}

pub fn parse_mac_addr(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(6u8)(input)
}

pub fn parse_message<'a>(input: &'a [u8]) -> FeigMessage {
    let raw = input.clone();
    match alt((parse_data_message, parse_keepalive))(input) {
        Ok(o) => o.1,
        Err(_e) => FeigMessage::Generic(raw.into()),
    }
}
