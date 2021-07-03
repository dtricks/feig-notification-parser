mod crc;
mod feig_types;
mod parser;

#[cfg(test)]
mod tests {
    use nom::value;

    use crate::{
        crc::*,
        feig_types::FeigMessage,
        parser::{check_message_code, get_message_length},
    };
    const KEEPALIVE_MESSAGE: [u8; 10] =
        [0x02u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x00, 0x00, 0x4b, 0x69];
    const FAULTY_KEEPALIVE_MESSAGE: [u8; 10] =
        [0x03u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x00, 0x00, 0x4b, 0x69];
    const FAULTY_CRC_KEEPALIVE_MESSAGE: [u8; 10] =
        [0x02u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x00, 0x00, 0x4b, 0x6a];
    fn get_data_message() -> Vec<u8> {
        hex::FromHex::from_hex(
            "020029002200a1020001001d84000e34000008740000000000000013280013df75001c9b070957370e",
        )
        .unwrap()
    }
    fn get_data_message_with_multiple_tags() -> Vec<u8> {
        hex::FromHex::from_hex(
            "020046002200a1020002001d84000e30002016040500000000000020180008a6f4001c9b070957001d84000e34000008740000000000000013280008a6f4001c9b070957ea8c",
        )
        .unwrap()
    }
    #[test]
    fn test_parse_keepalive() {
        let empty: &[u8] = &[];
        assert_eq!(
            crate::parser::parse_keepalive(&KEEPALIVE_MESSAGE),
            Ok((
                empty,
                FeigMessage::Keepalive {
                    raw: KEEPALIVE_MESSAGE.into(),
                    crc: 0x694b,
                    command_code: 0x6e,
                    com_adr: 0,
                    status: 0,
                    flags_a: 0,
                    flags_b: 0,
                    message_code: 2,
                    len: 10
                }
            ))
        );
    }
    #[test]
    fn test_parse_faulty_keepalive() {
        let empty: &[u8] = &[];
        assert_ne!(
            crate::parser::parse_keepalive(&FAULTY_KEEPALIVE_MESSAGE),
            Ok((
                empty,
                FeigMessage::Keepalive {
                    raw: KEEPALIVE_MESSAGE.into(),
                    crc: 0x694b,
                    command_code: 0x6e,
                    com_adr: 0,
                    status: 0,
                    flags_a: 0,
                    flags_b: 0,
                    message_code: 2,
                    len: 10
                }
            ))
        );
    }

    #[test]
    fn test_check_message_code() {
        let code: &[u8] = &[2];
        assert_eq!(
            check_message_code(&KEEPALIVE_MESSAGE),
            Ok((&KEEPALIVE_MESSAGE[1..], code))
        );
    }

    #[test]
    fn test_get_message_length() {
        assert_eq!(
            get_message_length(&KEEPALIVE_MESSAGE[1..]),
            Ok((&KEEPALIVE_MESSAGE[3..], 10))
        );
    }

    #[test]
    fn test_parse_data_message() {
        use crate::feig_types::TagRead;
        let empty: &[u8] = &[];
        assert_eq!(
            crate::parser::parse_data_message(&get_data_message()),
            Ok((
                empty,
                FeigMessage::Data {
                    raw: [
                        2, 0, 41, 0, 34, 0, 161, 2, 0, 1, 0, 29, 132, 0, 14, 52, 0, 0, 8, 116, 0,
                        0, 0, 0, 0, 0, 0, 19, 40, 0, 19, 223, 117, 0, 28, 155, 7, 9, 87, 55, 14
                    ]
                    .into(),
                    status: 0,
                    data: [TagRead {
                        record_len: 29,
                        transponder_type: 132,
                        idd_t: 0,
                        idd_len: 14,
                        serial_number: [52, 0, 0, 8, 116, 0, 0, 0, 0, 0, 0, 0, 19, 40].into(),
                        time: 1977553664,
                        mac: [0, 28, 155, 7, 9, 87].into()
                    }]
                    .into(),
                    command_code: 34,
                    message_code: 2,
                    com_adr: 0,
                    crc: 3639,
                    len: 41
                }
            ))
        );
    }

    #[test]
    fn test_crc() {
        let correct_crc = 0x694b;
        assert_eq!(calculate_crc(&KEEPALIVE_MESSAGE[0..8]), correct_crc)
    }

    #[test]
    fn test_parse_faulty_crc_keepalive() {
        let empty: &[u8] = &[];
        match crate::parser::parse_keepalive(&FAULTY_CRC_KEEPALIVE_MESSAGE) {
            Ok(_) => (),
            Err(x) => print!("{}", x),
        }
        assert_ne!(
            dbg!(crate::parser::parse_keepalive(
                &FAULTY_CRC_KEEPALIVE_MESSAGE
            )),
            Ok((
                empty,
                FeigMessage::Keepalive {
                    raw: KEEPALIVE_MESSAGE.into(),
                    crc: 0x694b,
                    command_code: 0x6e,
                    com_adr: 0,
                    status: 0,
                    flags_a: 0,
                    flags_b: 0,
                    message_code: 2,
                    len: 10
                }
            ))
        );
    }

    #[test]
    fn test_parse_unknown_message() {
        use crate::feig_types::TagRead;
        let fm = crate::parser::parse_message(&get_data_message());
        let expected = FeigMessage::Data {
            raw: [
                2, 0, 41, 0, 34, 0, 161, 2, 0, 1, 0, 29, 132, 0, 14, 52, 0, 0, 8, 116, 0, 0, 0, 0,
                0, 0, 0, 19, 40, 0, 19, 223, 117, 0, 28, 155, 7, 9, 87, 55, 14,
            ]
            .into(),
            status: 0,
            data: [TagRead {
                record_len: 29,
                transponder_type: 132,
                idd_t: 0,
                idd_len: 14,
                serial_number: [52, 0, 0, 8, 116, 0, 0, 0, 0, 0, 0, 0, 19, 40].into(),
                time: 1977553664,
                mac: [0, 28, 155, 7, 9, 87].into(),
            }]
            .into(),
            command_code: 34,
            message_code: 2,
            com_adr: 0,
            crc: 3639,
            len: 41,
        };
        assert_eq!(fm, expected);
    }
    #[test]
    fn test_parse_unknown_message_with_multiple_tags() {
        use crate::feig_types::TagRead;
        let fm = crate::parser::parse_message(&get_data_message_with_multiple_tags());
        dbg!(&fm);
        let expected = FeigMessage::Data {
            raw: [
                2, 0, 41, 0, 34, 0, 161, 2, 0, 1, 0, 29, 132, 0, 14, 52, 0, 0, 8, 116, 0, 0, 0, 0,
                0, 0, 0, 19, 40, 0, 19, 223, 117, 0, 28, 155, 7, 9, 87, 55, 14,
            ]
            .into(),
            status: 0,
            data: [TagRead {
                record_len: 29,
                transponder_type: 132,
                idd_t: 0,
                idd_len: 14,
                serial_number: [52, 0, 0, 8, 116, 0, 0, 0, 0, 0, 0, 0, 19, 40].into(),
                time: 1977553664,
                mac: [0, 28, 155, 7, 9, 87].into(),
            }]
            .into(),
            command_code: 34,
            message_code: 2,
            com_adr: 0,
            crc: 3639,
            len: 41,
        };
        assert_eq!(fm, expected);
    }
}
