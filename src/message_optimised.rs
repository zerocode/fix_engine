use crate::clock::Clock;
use crate::tag::*;
use std::sync::Arc;

pub struct FixMessage2 {
    pub header: [Option<FixTag>; 10],
    pub body: [Option<FixTag>; 10],
    pub trailer: [Option<FixTag>; 1],
}

impl FixMessage2 {
    pub(crate) fn encode(&mut self) -> String {

        // The message length must be specified in the BodyLength(9) field. The length must be calculatedpub pub  by counting the number of octets
        // in the message following the end of field delimiter (<SOH>) of BodyLength(9), up to and including the end of field delimiter (<SOH>)
        // of the field immediately preceding the CheckSum(10) field.
        // count each char in field, each char in tag_id, plus 2 for = and SOH, in header and body, excluding 8 and 9

        let body_length = calculate_body_length(self);
        let body_length_str = int_to_str_no_alloc(body_length, &mut [0u8; 16]).to_string();

        self.header[1] = Some(FixTag::BodyLength(body_length_str));

        // render to string
        let msg_str = self.header.iter().chain(self.body.iter())
            .filter_map(|tag|
                tag.as_ref()
                    .map(|t| { [t.tag_id(), "=", t.value().as_str(), "\x01"].concat() })
            ).collect::<String>();

        // add checksum
        let checksum = int_to_str_no_alloc(calculate_checksum(msg_str.clone()), &mut [0u8; 16]).to_string();
        self.trailer[0] = Some(FixTag::Checksum(checksum.clone()));

        [msg_str, "10=".to_string(), checksum, "\x01".to_string()].concat()
    }
}

fn calculate_body_length(message: &FixMessage2) -> usize {
    message.header.iter().chain(message.body.iter())
        .filter_map(|tag|
            tag.as_ref()
                .filter(|t| t.tag_id() != "8" && t.tag_id() != "9") // Exclude 8 and 9
                .map(|t| t.value().len() + t.tag_id().len() + 2)
        ).sum::<usize>()
}

fn calculate_checksum(fix_str: String) -> usize {
    fix_str.as_bytes().iter().map(|&b| b as usize).sum::<usize>() % 256
}

fn int_to_str_no_alloc(n: usize, buffer: &mut [u8]) -> &str {
    // Start from the end of the buffer and work backwards
    let mut pos = buffer.len();

    // Special case for zero
    if n == 0 {
        pos -= 1;
        buffer[pos] = b'0'; // Store '0' in the buffer
    } else {
        let mut num = n;

        // Convert each digit to its ASCII representation
        while num > 0 {
            pos -= 1;
            buffer[pos] = b'0' + (num % 10) as u8; // Get the last digit and convert it to ASCII
            num /= 10; // Remove the last digit
        }
    }

    // Convert the used part of the buffer to a &str
    core::str::from_utf8(&buffer[pos..]).unwrap()
}

impl FixMessage2 {
    pub fn new() -> Self {
        Self {
            header: [const { None }; 10],
            body: [const { None }; 10],
            trailer: [None; 1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock;
    use crate::tag::BeginString;
    use crate::tag::CompID;
    use crate::tag::FixTag;
    use crate::tag::MsgType;
    use std::sync::Arc;

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> String {
            "20231016-12:30:00.123".to_string()
        }
    }

    fn create_fixed_clock() -> Arc<dyn Clock> {
        Arc::new(FixedClock)
    }

    #[test]
    fn test_encode_decode_a_fix_message() {
        let mut msg = create_test_message();
        let encoded_message = msg.encode();
        println!("{}", encoded_message)
        // let decoded_message = FixMessage::decode(&encoded_message).unwrap();
        //
        // // Verify header fields
        // assert_eq!(decoded_message.header.get("8").unwrap(), "FIX.4.4");
        // assert_eq!(decoded_message.header.get("35").unwrap(), "A");
        // assert_eq!(decoded_message.header.get("49").unwrap(), "SENDER");
        // assert_eq!(decoded_message.header.get("56").unwrap(), "TARGET");
        // assert_eq!(decoded_message.header.get("34").unwrap(), "1");
        //
        // // Verify body fields
        // assert_eq!(decoded_message.body.get("98").unwrap(), "0");
        // assert_eq!(decoded_message.body.get("108").unwrap(), "30");
        //
        // // Verify the checksum field
        // assert!(decoded_message.trailer.contains_key("10"));
    }

    fn create_test_message() -> FixMessage2 {
        let fixed_clock = create_fixed_clock();
        let mut msg = FixMessage2::new();
        msg.header[0] = Some(FixTag::BeginString(BeginString::Fix4_2));
        msg.header[2] = Some(FixTag::MsgType(MsgType::Logon));
        msg.header[3] = Some(FixTag::MsgSeqNum("1".to_string()));
        msg.header[4] = Some(FixTag::SendingTime(fixed_clock.now()));
        msg.header[5] = Some(FixTag::SenderCompID(CompID("SENDER".to_string())));
        msg.header[6] = Some(FixTag::TargetCompID(CompID("TARGET".to_string())));
        msg.body[0] = Some(FixTag::Symbol("BTCUSDT".to_string()));
        msg
    }

    #[test]
    fn test_calculate_checksum_correctly() {
        let message_without_checksum = "8=FIX.4.4\x019=59\x0135=A\x0149=SENDER\x0156=TARGET\x0134=1\x0152=20231016-12:30:00.123\x0198=0\x01108=30\x01".to_string();
        let calculated_checksum = calculate_checksum(message_without_checksum);
        let expected_checksum = 119;

        assert_eq!(calculated_checksum, expected_checksum);
    }

    #[test]
    fn test_calculate_body_length_correctly() {
        let message = create_test_message();
        let calculated_checksum = calculate_body_length(&message);
        let expected_checksum = 66;

        assert_eq!(calculated_checksum, expected_checksum);
    }
}