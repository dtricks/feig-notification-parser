mod crc;
pub mod feig_types;
pub mod parser;

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]

    use crate::{
        crc::*,
        feig_types::{FeigMessage, TransponderType},
        parser::{check_message_code, get_message_length, parse_message},
    };
    const KEEPALIVE_MESSAGE: [u8; 10] =
        [0x02u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x00, 0x00, 0x4b, 0x69];
    const FAULTY_KEEPALIVE_MESSAGE: [u8; 10] =
        [0x03u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x00, 0x00, 0x4b, 0x69];
    const FAULTY_CRC_KEEPALIVE_MESSAGE: [u8; 10] =
        [0x02u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x00, 0x00, 0x4b, 0x6a];
    const KEEPALIVE_MESSAGE_WITH_ALL_ERRORS: [u8; 10] =
        [0x02u8, 0x00, 0x0a, 0x00, 0x6e, 0x00, 0x96, 0x04, 0x4b, 0x69];
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
        assert!(match crate::parser::parse_message(&KEEPALIVE_MESSAGE) {
            FeigMessage::Data { .. } => false,
            FeigMessage::Generic(_) => false,
            FeigMessage::Keepalive { .. } => true,
        });
    }
    #[test]
    fn test_parse_faulty_keepalive() {
        assert!(
            match crate::parser::parse_message(&FAULTY_KEEPALIVE_MESSAGE) {
                FeigMessage::Data { .. } => false,
                FeigMessage::Generic(_) => false,
                FeigMessage::Keepalive { message_code, .. } => !(message_code == 2),
            }
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
        assert!(match crate::parser::parse_message(&get_data_message()) {
            FeigMessage::Data { .. } => true,
            FeigMessage::Generic(_) => false,
            FeigMessage::Keepalive { .. } => false,
        });
    }

    #[test]
    fn test_crc() {
        let correct_crc = 0x694b;
        assert_eq!(
            calculate_crc(&KEEPALIVE_MESSAGE[0..KEEPALIVE_MESSAGE.len() - 2]),
            correct_crc
        )
    }

    #[test]
    fn test_parse_faulty_crc_keepalive() {
        let empty: &[u8] = &[];
        assert!(
            match crate::parser::parse_message(&FAULTY_CRC_KEEPALIVE_MESSAGE) {
                FeigMessage::Data { .. } => false,
                FeigMessage::Generic(_) => false,
                FeigMessage::Keepalive { correct_crc, .. } => correct_crc == false,
            }
        );
    }

    #[test]
    fn test_parse_unknown_message() {
        assert!(match crate::parser::parse_message(&get_data_message()) {
            FeigMessage::Data { .. } => true,
            FeigMessage::Generic(_) => false,
            FeigMessage::Keepalive { .. } => false,
        });
    }
    #[test]
    fn test_parse_unknown_message_with_multiple_tags() {
        assert!(
            match parse_message(&get_data_message_with_multiple_tags()) {
                FeigMessage::Data {
                    raw,
                    status,
                    tags,
                    command_code,
                    message_code,
                    com_adr,
                    crc,
                    len,
                    correct_crc,
                } => {
                    dbg!(command_code);
                    true
                }
                FeigMessage::Generic(_) => false,
                FeigMessage::Keepalive { .. } => false,
            }
        );
    }
    #[test]
    fn test_keepalive_error_parsing() {
        assert!(match parse_message(&KEEPALIVE_MESSAGE_WITH_ALL_ERRORS) {
            FeigMessage::Keepalive {
                flag_temp_alarm,
                flag_false_power,
                flag_wrong_antenna_impedance,
                flag_dc_power_error,
                flag_noise,
                ..
            } =>
                dbg!(flag_temp_alarm)
                    && dbg!(flag_false_power)
                    && dbg!(flag_wrong_antenna_impedance)
                    && dbg!(flag_noise)
                    && dbg!(flag_dc_power_error),
            _ => false,
        })
    }
    #[test]
    fn test_transponder_type_parsing() {
        assert!(match parse_message(&get_data_message()) {
            FeigMessage::Data { tags, .. } => match tags.get(0).unwrap().transponder_type {
                TransponderType::Iso18000_3M3 => true,
                _ => false,
            },
            _ => false,
        })
    }
    #[test]
    fn test_serde() {
        use serde_json;
        let fm = parse_message(&get_data_message());
        let serde = serde_json::ser::to_string_pretty(&fm).unwrap();
        println!("{}", serde);
        assert!(serde.contains("Iso18000_3M3"))
    }
}
