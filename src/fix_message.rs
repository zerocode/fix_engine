use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter, Write};
use std::sync::Arc;

pub const SOH: char = '\x01';
const CHECKSUM_TAG: &str = "10";
const REQUIRED_HEADER_FIELDS: [&str; 7] = ["8", "9", "35", "49", "56", "34", "52"];

// Trait defining a Clock that provides the current time as a UTC string.

pub trait Clock: Send + Sync {
    fn now(&self) -> String;
}

// RealClock provides the actual system time.
#[derive(Debug)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> String {
        let now = chrono::Utc::now();
        format!("{}", now.format("%Y%m%d-%H:%M:%S%.3f"))
    }
}

// Helper function for calculating the checksum (mod 256 sum of all characters)
fn calculate_checksum(fix_str: &str) -> String {
    let sum: u32 = fix_str.as_bytes().iter().map(|&b| b as u32).sum();
    format!("{:03}", sum % 256)
}


pub struct FixMessage {
    pub header: HashMap<String, String>,
    pub body: HashMap<String, String>,
    pub trailer: HashMap<String, String>,
    clock: Arc<dyn Clock>, // Use Arc to make it shareable across threads
}

impl Debug for FixMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixMessage")
            .field("header", &self.header)
            .field("body", &self.body)
            .field("trailer", &self.trailer)
            .finish() // Exclude the `clock` field
    }
}

impl FixMessage {
    pub fn new(clock: Arc<dyn Clock>) -> FixMessage {
        FixMessage {
            header: HashMap::new(),
            body: HashMap::new(),
            trailer: HashMap::new(),
            clock,
        }
    }

    // Encodes a FIX message into a string, with correct header, body, trailer, and checksum
    pub fn encode(&mut self) -> String {
        // Ensure mandatory fields are populated
        if !self.header.contains_key("8") {
            self.header.insert("8".to_string(), "FIX.4.4".to_string());
        }
        if !self.header.contains_key("52") {
            self.header.insert("52".to_string(), self.clock.now());
        }

        // Construct the body string
        let mut fix_body = String::new();
        for (tag, value) in &self.body {
            write!(fix_body, "{}={}{}", tag, value, SOH).unwrap();
        }

        // Calculate BodyLength (all characters after BodyLength tag up to and including SOH before checksum)
        let body_length = fix_body.len() + self.header.iter().map(|(k, v)| k.len() + v.len() + 2).sum::<usize>() + 3;

        self.header.insert("9".to_string(), body_length.to_string()); // BodyLength tag

        // Construct the header string in correct order
        let mut fix_header = String::new();
        for &tag in REQUIRED_HEADER_FIELDS.iter() {
            if let Some(value) = self.header.get(tag) {
                write!(fix_header, "{}={}{}", tag, value, SOH).unwrap();
            }
        }
        for (tag, value) in &self.header {
            if !REQUIRED_HEADER_FIELDS.contains(&tag.as_str()) {
                write!(fix_header, "{}={}{}", tag, value, SOH).unwrap();
            }
        }

        // Combine header and body
        let message_without_checksum = format!("{}{}", fix_header, fix_body);

        // Calculate the checksum and append it
        let checksum = calculate_checksum(&message_without_checksum);
        self.trailer.insert(CHECKSUM_TAG.to_string(), checksum);

        // Construct the trailer
        let mut fix_trailer = String::new();
        for (tag, value) in &self.trailer {
            write!(fix_trailer, "{}={}{}", tag, value, SOH).unwrap();
        }

        // Complete FIX message
        format!("{}{}{}", fix_header, fix_body, fix_trailer)
    }

    // Decodes a FIX message from a string, extracting fields into the header, body, and trailer
    pub fn decode(fix_str: &str, clock: Arc<dyn Clock>) -> Result<FixMessage, &'static str> {
        let mut message = FixMessage::new(clock);
        let mut trailer_started = false;

        // Split by the SOH delimiter
        let tags: Vec<&str> = fix_str.split(SOH).filter(|&x| !x.is_empty()).collect();

        for tag_value in tags {
            let mut parts = tag_value.split('=');
            let tag = parts.next().unwrap();
            let value = parts.next().ok_or("Invalid FIX message format")?;

            // Determine if we're in the trailer section
            if tag == CHECKSUM_TAG {
                trailer_started = true;
            }

            // Fill the appropriate section based on the tag
            if trailer_started {
                message.trailer.insert(tag.to_string(), value.to_string());
            } else if REQUIRED_HEADER_FIELDS.contains(&tag) || tag == "9" {
                message.header.insert(tag.to_string(), value.to_string());
            } else {
                message.body.insert(tag.to_string(), value.to_string());
            }
        }

        // Validate checksum
        let received_checksum = message.trailer.get(CHECKSUM_TAG).ok_or("Checksum missing")?.clone();
        let checksum_input: String = fix_str.chars().take(fix_str.len() - received_checksum.len() - 4).collect(); // exclude "10=xxx" from checksum calculation
        let calculated_checksum = calculate_checksum(&checksum_input);
        if received_checksum != calculated_checksum {
            return Err("Invalid checksum");
        }

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // FixedClock for tests
    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> String {
            "20231016-12:30:00.123".to_string() // Fixed timestamp for testing
        }
    }

    #[test]
    fn test_fix_message_encode_decode() {
        let fixed_clock = Arc::new(FixedClock);
        let mut msg = FixMessage::new(fixed_clock.clone());
        msg.header.insert("8".to_string(), "FIX.4.4".to_string());  // BeginString
        msg.header.insert("35".to_string(), "A".to_string());       // MsgType (Logon)
        msg.header.insert("49".to_string(), "SENDER".to_string());  // SenderCompID
        msg.header.insert("56".to_string(), "TARGET".to_string());  // TargetCompID
        msg.header.insert("34".to_string(), "1".to_string());       // MsgSeqNum
        msg.header.insert("52".to_string(), fixed_clock.now());     // SendingTime (fixed for testing)
        msg.body.insert("98".to_string(), "0".to_string());         // EncryptMethod (example field in body)
        msg.body.insert("108".to_string(), "30".to_string());       // HeartBtInt (example field in body)

        let encoded = msg.encode();
        println!("Encoded message: {}", encoded);

        let decoded = FixMessage::decode(&encoded, fixed_clock.clone()).unwrap();

        // Check header fields
        assert_eq!(decoded.header.get("8").unwrap(), "FIX.4.4");
        assert_eq!(decoded.header.get("35").unwrap(), "A");
        assert_eq!(decoded.header.get("49").unwrap(), "SENDER");
        assert_eq!(decoded.header.get("56").unwrap(), "TARGET");
        assert_eq!(decoded.header.get("34").unwrap(), "1");

        // Check body fields
        assert_eq!(decoded.body.get("98").unwrap(), "0");
        assert_eq!(decoded.body.get("108").unwrap(), "30");
    }

    #[test]
    fn test_invalid_checksum() {
        let fixed_clock = Arc::new(FixedClock);
        let invalid_message = "8=FIX.4.4\x019=59\x0135=A\x0149=SENDER\x0156=TARGET\x0134=1\x0152=20231016-12:30:00.123\x0198=0\x01108=30\x0110=999\x01";
        let result = FixMessage::decode(invalid_message, fixed_clock);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Invalid checksum");
    }
}