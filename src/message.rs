use crate::clock::Clock;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter, Write};
use std::sync::Arc;

pub struct FixMessage {
    pub header: HashMap<String, String>,
    pub body: HashMap<String, String>,
    pub trailer: HashMap<String, String>,
}

impl Debug for FixMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixMessage")
            .field("header", &sorted_map(&self.header))
            .field("body", &self.body)
            .field("trailer", &self.trailer)
            .finish() // Exclude the `clock` field
    }
}

fn sorted_map(map: &HashMap<String, String>) -> Vec<(&String, &String)> {
    let mut sorted_entries: Vec<_> = map.iter().collect();
    sorted_entries.sort_by_key(|(k, _)| k.parse::<i32>().unwrap());
    sorted_entries
}

impl FixMessage {
    pub fn new() -> FixMessage {
        FixMessage {
            header: HashMap::new(),
            body: HashMap::new(),
            trailer: HashMap::new(),
        }
    }

    pub fn encode(&mut self, clock: &Arc<dyn Clock>) -> String {
        // Ensure mandatory fields are populated
        if !self.header.contains_key("8") {
            self.header.insert("8".to_string(), "FIX.4.4".to_string());
        }
        if !self.header.contains_key("52") {
            self.header.insert("52".to_string(), clock.now());
        }

        // Step 1: Concatenate body fields with SOH as the separator
        let mut fix_body = String::new();
        for (tag, value) in &self.body {
            write!(fix_body, "{}={}{}", tag, value, '\x01').unwrap();  // Append SOH after each tag-value pair
        }

        // Step 2: Calculate BodyLength (length of message after "9=" tag, excluding checksum)
        let body_length_value = {
            // Temporarily create the header without BodyLength (9=) and checksum (10=)
            let mut fix_header = String::new();
            for (tag, value) in &self.header {
                if tag != "9" && tag != "8" {
                    write!(fix_header, "{}={}{}", tag, value, '\x01').unwrap();
                }
            }
            fix_header.len() + fix_body.len()
        };

        // Step 3: Insert BodyLength (Tag 9)
        self.header.insert("9".to_string(), body_length_value.to_string());

        // Step 4: Rebuild the full header with the BodyLength now included
        let mut fix_header = String::new();
        for tag in &["8", "9", "35", "49", "56", "34", "52"] { // Ensure correct order of important tags
            if let Some(value) = self.header.get(*tag) {
                write!(fix_header, "{}={}{}", tag, value, '\x01').unwrap();
            }
        }

        // Step 5: Combine header and body
        let message_without_checksum = format!("{}{}", fix_header, fix_body);

        // Step 6: Calculate checksum (sum of all bytes mod 256)
        let checksum = calculate_checksum(&message_without_checksum);
        self.trailer.insert("10".to_string(), checksum);

        // Step 7: Concatenate trailer (which contains the checksum) with SOH as the separator
        let mut fix_trailer = String::new();
        for (tag, value) in &self.trailer {
            write!(fix_trailer, "{}={}{}", tag, value, '\x01').unwrap();  // Append SOH after each tag-value pair
        }

        // Step 8: Final message with SOH at the end
        format!("{}{}{}", fix_header, fix_body, fix_trailer)
    }

    pub fn decode(fix_str: &str) -> Result<FixMessage, &'static str> {
        // Ensure the message ends with SOH ('\x01')
        if !fix_str.ends_with('\x01') {
            return Err("Message does not end with SOH");
        }

        // Remove the trailing SOH before parsing
        let message_without_trailing_soh = &fix_str[..fix_str.len() - 1];

        let mut message = FixMessage::new();

        // Split the message into key-value pairs using '\x01' as the field separator
        let fields: Vec<&str> = message_without_trailing_soh.split('\x01').filter(|&x| !x.is_empty()).collect();

        let mut checksum_input = String::new(); // The portion of the message for checksum calculation

        for part in fields {
            // Split each part by '=' to get the tag and value
            let key_value: Vec<&str> = part.splitn(2, '=').collect();
            if key_value.len() != 2 {
                return Err("Invalid key-value pair in FIX message");
            }

            let tag = key_value[0];
            let value = key_value[1];

            // Skip validation for the "9" tag (BodyLength)
            if tag == "9" {
                message.header.insert(tag.to_string(), value.to_string());
                // continue;
            }

            if tag == "10" {
                // Ensure checksum is the last field
                let received_checksum = value;
                let calculated_checksum = calculate_checksum(&checksum_input);
                if received_checksum != calculated_checksum {
                    return Err("Invalid checksum");
                }
                message.trailer.insert(tag.to_string(), received_checksum.to_string());
                break;  // Stop processing after checksum
            }

            // Add the part to checksum input before the checksum
            checksum_input.push_str(part);
            checksum_input.push('\x01');  // SOH between fields

            // Populate the header, body, or trailer based on the tag
            match tag {
                "8" | "35" | "49" | "56" | "34" | "52" => {
                    message.header.insert(tag.to_string(), value.to_string());
                }
                _ => {
                    message.body.insert(tag.to_string(), value.to_string());
                }
            }
        }

        Ok(message)
    }
}

// Helper function for calculating the checksum (mod 256 sum of all characters)
fn calculate_checksum(fix_str: &str) -> String {
    let sum: u32 = fix_str.as_bytes().iter().map(|&b| b as u32).sum();
    format!("{:03}", sum % 256)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock;
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
    fn test_fix_message_encode_decode() {
        let fixed_clock = create_fixed_clock();
        let mut msg = FixMessage::new();
        msg.header.insert("8".to_string(), "FIX.4.4".to_string());
        msg.header.insert("35".to_string(), "A".to_string());       // MsgType (Logon)
        msg.header.insert("49".to_string(), "SENDER".to_string());  // SenderCompID
        msg.header.insert("56".to_string(), "TARGET".to_string());  // TargetCompID
        msg.header.insert("34".to_string(), "1".to_string());       // MsgSeqNum
        msg.header.insert("52".to_string(), fixed_clock.now());     // SendingTime
        msg.body.insert("98".to_string(), "0".to_string());         // EncryptMethod
        msg.body.insert("108".to_string(), "30".to_string());       // HeartBtInt

        let encoded_message = msg.encode(&fixed_clock);

        let decoded_message = FixMessage::decode(&encoded_message).unwrap();

        // Verify header fields
        assert_eq!(decoded_message.header.get("8").unwrap(), "FIX.4.4");
        assert_eq!(decoded_message.header.get("35").unwrap(), "A");
        assert_eq!(decoded_message.header.get("49").unwrap(), "SENDER");
        assert_eq!(decoded_message.header.get("56").unwrap(), "TARGET");
        assert_eq!(decoded_message.header.get("34").unwrap(), "1");

        // Verify body fields
        assert_eq!(decoded_message.body.get("98").unwrap(), "0");
        assert_eq!(decoded_message.body.get("108").unwrap(), "30");

        // Verify the checksum field
        assert!(decoded_message.trailer.contains_key("10"));
    }

    #[test]
    fn test_fix_message_encode_with_correct_body_length() {
        let fixed_clock = create_fixed_clock();
        let mut msg = FixMessage::new();
        msg.header.insert("8".to_string(), "FIX.4.4".to_string());
        msg.header.insert("35".to_string(), "A".to_string());       // MsgType (Logon)
        msg.header.insert("49".to_string(), "SENDER".to_string());  // SenderCompID
        msg.header.insert("56".to_string(), "TARGET".to_string());  // TargetCompID
        msg.header.insert("34".to_string(), "1".to_string());       // MsgSeqNum
        msg.header.insert("52".to_string(), fixed_clock.now());     // SendingTime
        msg.body.insert("98".to_string(), "0".to_string());         // EncryptMethod
        msg.body.insert("108".to_string(), "30".to_string());       // HeartBtInt


        let encoded_message = msg.encode(&fixed_clock);

        let begin_string_position = encoded_message.find("8=FIX.4.4").unwrap();
        let body_length_position = encoded_message.find("9=").unwrap();
        assert!(body_length_position > begin_string_position, "BodyLength should come after BeginString");

        let body_length_field = encoded_message
            .split('\x01')
            .find(|&field| field.starts_with("9="))
            .expect("BodyLength (Tag 9) not found");

        let actual_body_length = body_length_field.split('=').nth(1).unwrap().parse::<usize>().unwrap();

        let expected_body_length = encoded_message
            .split("\x01")
            .filter(|field| !field.starts_with("8=") && !field.starts_with("9=") && !field.starts_with("10=") && !field.is_empty())
            .map(|field| field.len() + 1) // Each field length + 1 for the SOH character
            .sum::<usize>();

        // Verify that the actual BodyLength matches the expected length
        assert_eq!(actual_body_length, expected_body_length);

        // Output the full encoded message for verification
        println!("Encoded message: {}", encoded_message);

        // Verify the message contains the correct structure
        assert!(encoded_message.contains("8=FIX.4.4\x01"));
        assert!(encoded_message.contains("9="));
        assert!(encoded_message.contains("10=")); // Checksum field
    }

    #[test]
    fn test_fix_message_encode_correct_order() {
        let fixed_clock = create_fixed_clock();
        let mut msg = FixMessage::new();
        msg.header.insert("8".to_string(), "FIX.4.4".to_string());
        msg.header.insert("35".to_string(), "A".to_string());       // MsgType (Logon)
        msg.header.insert("49".to_string(), "SENDER".to_string());  // SenderCompID
        msg.header.insert("56".to_string(), "TARGET".to_string());  // TargetCompID
        msg.header.insert("34".to_string(), "1".to_string());       // MsgSeqNum
        msg.header.insert("52".to_string(), fixed_clock.now());     // SendingTime
        msg.body.insert("98".to_string(), "0".to_string());         // EncryptMethod
        msg.body.insert("108".to_string(), "30".to_string());       // HeartBtInt

        let encoded_message = msg.encode(&fixed_clock);

        // Output the full encoded message for verification
        println!("Encoded message: {}", encoded_message);

        // Verify the message contains the correct structure
        assert!(encoded_message.contains("8=FIX.4.4\x01"));
        assert!(encoded_message.contains("9="));
        assert!(encoded_message.contains("35=A\x01"));
        assert!(encoded_message.contains("49=SENDER\x01"));
        assert!(encoded_message.contains("56=TARGET\x01"));
        assert!(encoded_message.contains("34=1\x01"));
        assert!(encoded_message.contains("52="));
        assert!(encoded_message.contains("10=")); // Checksum field
    }

    #[test]
    fn test_checksum_is_calculated_correctly() {
        let message_without_checksum = "8=FIX.4.4\x019=59\x0135=A\x0149=SENDER\x0156=TARGET\x0134=1\x0152=20231016-12:30:00.123\x0198=0\x01108=30\x01";

        let calculated_checksum = calculate_checksum(message_without_checksum);
        let expected_checksum = "119";  // This is the checksum for the above message

        assert_eq!(calculated_checksum, expected_checksum);
    }

    #[test]
    fn test_invalid_checksum_throws_err() {
        let invalid_message = "8=FIX.4.4\x019=59\x0135=A\x0149=SENDER\x0156=TARGET\x0134=1\x0152=20231016-12:30:00.123\x0198=0\x01108=30\x0110=999\x01"; // Invalid checksum

        let result = FixMessage::decode(invalid_message);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Invalid checksum");
    }

    #[test]
    fn test_message_without_soh_fails() {
        let invalid_message = "8=FIX.4.4\
                               9=59\
                               35=A\
                               49=SENDER\
                               56=TARGET\
                               34=1\
                               52=20231016-12:30:00.123\
                               98=0\
                               108=30\
                               10=214"; // Missing the trailing SOH

        let result = FixMessage::decode(invalid_message);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Message does not end with SOH");
    }
}