#![allow(dead_code)]

use nom::branch::alt;
use nom::bytes::complete::take;
//use nom::complete::take;
use nom::IResult;
//use nom::error::Error;
use nom::number::complete::{be_u16, le_u16};
use nom::number::complete::{be_u32, le_u8};
use nom::Err::Error;
use thiserror::Error as ThisError;
//use nom::IResult;

use crate::feig_types::FeigMessage;
use crate::feig_types::TagRead;

#[derive(Debug, PartialEq, ThisError)]
pub enum FeigError {
    //<I> {
    #[error("WrongCrc")]
    WrongCrc,
    #[error("WrongMessageCode")]
    WrongMessageCode,
    //Nom(I, ErrorKind),
}

pub fn parse_keepalive(input: &[u8]) -> IResult<&[u8], FeigMessage> {
    let raw = input.clone();
    let (input, message_code) = check_message_code(input)?;
    let (input, len) = get_message_length(input)?;
    let (input, com_adr) = get_com_adr(input)?;
    let (input, command_code) = get_command_code(input)?;
    let (input, status) = get_status(input)?;
    let (input, keepalive_data) = get_keepalive_data(input)?;
    let (input, crc) = get_crc(input)?;
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
        },
    ))
}

pub fn check_message_code(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(1u8)(input)
}

pub fn check_message_code_equals_two(input: &[u8]) -> IResult<&[u8], bool, FeigError> {
    if input[0] == 2 {
        Ok((input, true))
    } else {
        Err(Error(FeigError::WrongMessageCode))
    }
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
    Ok((
        input,
        FeigMessage::Data {
            raw: raw.into(),
            status: status[0],
            data: tags,
            command_code: command_code[0],
            message_code: message_code[0],
            com_adr: com_adr[0],
            crc,
            len,
        },
    ))
}

pub fn parse_tag_read(input: &[u8]) -> IResult<&[u8], TagRead> {
    // 02 00 01 | 001d 84 00 0e 3400000874000000000000001328 14207e9d 001c9b070957 8ab2
    // nf st cc | coun tt it len                         idd     time          mac  crc
    //count u16 0001
    let (input, record_len) = get_message_length(input)?;
    //transponder type
    let (input, tt) = le_u8(input)?;
    //idd type
    let (input, it) = le_u8(input)?;
    //idd len
    let (input, idd_len) = le_u8(input)?;
    //idd
    let (input, idd) = take(idd_len)(input)?;
    //time
    let (input, time) = be_u32(input)?;
    //mac-addr
    let (input, mac) = take(6u8)(input)?;
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
        Err(e) => FeigMessage::Generic(raw.into()),
    }
}
