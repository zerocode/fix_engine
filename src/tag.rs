pub const SOH: char = '\x01';
const CHECKSUM_TAG: &str = "10";
const REQUIRED_HEADER_FIELDS: [&str; 7] = ["8", "9", "35", "49", "56", "34", "52"];

trait FixField {
    fn tag_id(&self) -> u32;
    fn field_name(&self) -> &'static str;
    fn value(&self) -> &'static str; // Use &'static str to avoid heap allocation.
}

#[derive(Debug, Clone)]
struct CompID(&'static str); // Use &'static str instead of String.

impl CompID {
    fn new(id: &'static str) -> Self {
        CompID(id) // No allocation, just a reference to a static string.
    }
}

impl FixField for CompID {
    fn tag_id(&self) -> u32 {
        49
    }

    fn field_name(&self) -> &'static str {
        "SenderCompID"
    }

    fn value(&self) -> &'static str {
        self.0 // Return a reference to a static string.
    }
}

#[derive(Debug, Clone)]
enum PossDupFlag {
    Yes,
    No,
}

impl PossDupFlag {
    fn from_str(value: &str) -> Result<Self, &'static str> {
        match value {
            "Y" => Ok(PossDupFlag::Yes),
            "N" => Ok(PossDupFlag::No),
            _ => Err("Invalid PossDupFlag value"), // No allocation, just a static error message.
        }
    }
}

impl FixField for PossDupFlag {
    fn tag_id(&self) -> u32 {
        43
    }

    fn field_name(&self) -> &'static str {
        "PossDupFlag"
    }

    fn value(&self) -> &'static str {
        match self {
            PossDupFlag::Yes => "Y",
            PossDupFlag::No => "N",
        }
    }
}

#[derive(Debug, Clone)]
enum BeginString {
    Fix4_2,
    Fix4_4,
}

impl FixField for BeginString {
    fn tag_id(&self) -> u32 {
        8
    }

    fn field_name(&self) -> &'static str {
        "BeginString"
    }

    fn value(&self) -> &'static str {
        match self {
            BeginString::Fix4_2 => "FIX.4.2",
            BeginString::Fix4_4 => "FIX.4.4",
        }
    }
}

#[derive(Debug, Clone)]
enum MsgType {
    Heartbeat,
    TestRequest,
    ResendRequest,
    Reject,
    SequenceReset,
    Logout,
    ExecutionReport,
    OrderCancelReject,
    Logon,
    News,
    SecurityDefinitionRequest,
    OrderSingle,
    SecurityDefinition,
    SecurityStatusRequest,
    SecurityStatus,
    OrderCancelRequest,
    OrderCancelReplaceRequest,
    OrderStatusRequest,
    DontKnowTrade,
    QuoteRequest,
    MarketDataRequest,
    MarketDataSnapshotFullRefresh,
    MarketDataIncrementalRefresh,
    MarketDataRequestReject,
    TradeCaptureReportRequest,
    TradeCaptureReport,
    TradeCaptureReportRequestAck,
}

impl FixField for MsgType {
    fn tag_id(&self) -> u32 {
        35
    }

    fn field_name(&self) -> &'static str {
        "MsgType"
    }

    fn value(&self) -> &'static str {
        match self {
            MsgType::Heartbeat => "0",
            MsgType::TestRequest => "1",
            MsgType::ResendRequest => "2",
            MsgType::Reject => "3",
            MsgType::SequenceReset => "4",
            MsgType::Logout => "5",
            MsgType::ExecutionReport => "8",
            MsgType::OrderCancelReject => "9",
            MsgType::Logon => "A",
            MsgType::News => "B",
            MsgType::SecurityDefinitionRequest => "c",
            MsgType::OrderSingle => "D",
            MsgType::SecurityDefinition => "d",
            MsgType::SecurityStatusRequest => "e",
            MsgType::SecurityStatus => "f",
            MsgType::OrderCancelRequest => "F",
            MsgType::OrderCancelReplaceRequest => "G",
            MsgType::OrderStatusRequest => "H",
            MsgType::DontKnowTrade => "Q",
            MsgType::QuoteRequest => "R",
            MsgType::MarketDataRequest => "V",
            MsgType::MarketDataSnapshotFullRefresh => "W",
            MsgType::MarketDataIncrementalRefresh => "X",
            MsgType::MarketDataRequestReject => "Y",
            MsgType::TradeCaptureReportRequest => "AD",
            MsgType::TradeCaptureReport => "AE",
            MsgType::TradeCaptureReportRequestAck => "AQ",
        }
    }
}

#[derive(Debug, Clone)]
enum FixTag {
    BeginString(BeginString),
    MsgType(MsgType),
    BodyLength(&'static str),
    SenderCompID(CompID),
    TargetCompID(CompID),
    SenderSubID(&'static str), // Use &'static str for fixed-size strings.
    TargetSubID(&'static str), // Use &'static str for fixed-size strings.
    OnBehalfOfSubID(&'static str), // Use &'static str for fixed-size strings.
    MsgSeqNum(&'static str),
    SenderLocationID(&'static str), // Use &'static str for fixed-size strings.
    PossDupFlag(PossDupFlag),
    OrigSendingTime(&'static str), // Use &'static str for fixed-size strings.
    SendingTime(&'static str), // Use &'static str for fixed-size strings.
}

impl FixField for FixTag {
    fn tag_id(&self) -> u32 {
        match self {
            FixTag::BeginString(f) => f.tag_id(),
            FixTag::MsgType(f) => f.tag_id(),
            FixTag::BodyLength(_) => 9,
            FixTag::SenderCompID(f) => f.tag_id(),
            FixTag::TargetCompID(_) => 56,
            FixTag::SenderSubID(_) => 50,
            FixTag::TargetSubID(_) => 57,
            FixTag::OnBehalfOfSubID(_) => 116,
            FixTag::MsgSeqNum(_) => 34,
            FixTag::SenderLocationID(_) => 142,
            FixTag::PossDupFlag(f) => f.tag_id(),
            FixTag::OrigSendingTime(_) => 122,
            FixTag::SendingTime(_) => 52,
        }
    }

    fn field_name(&self) -> &'static str {
        match self {
            FixTag::BeginString(f) => f.field_name(),
            FixTag::MsgType(f) => f.field_name(),
            FixTag::BodyLength(_) => "BodyLength",
            FixTag::SenderCompID(f) => f.field_name(),
            FixTag::TargetCompID(_) => "TargetCompID",
            FixTag::SenderSubID(_) => "SenderSubID",
            FixTag::TargetSubID(_) => "TargetSubID",
            FixTag::OnBehalfOfSubID(_) => "OnBehalfOfSubID",
            FixTag::MsgSeqNum(_) => "MsgSeqNum",
            FixTag::SenderLocationID(_) => "SenderLocationID",
            FixTag::PossDupFlag(f) => f.field_name(),
            FixTag::OrigSendingTime(_) => "OrigSendingTime",
            FixTag::SendingTime(_) => "SendingTime",
        }
    }

    fn value(&self) -> &'static str {
        match self {
            FixTag::BeginString(f) => f.value(),
            FixTag::MsgType(f) => f.value(),
            FixTag::BodyLength(length) => length, // Use the abstracted function.
            FixTag::SenderCompID(f) => f.value(),
            FixTag::TargetCompID(f) => f.value(),
            FixTag::SenderSubID(sub_id) => sub_id, // Return static reference.
            FixTag::TargetSubID(sub_id) => sub_id, // Return static reference.
            FixTag::OnBehalfOfSubID(sub_id) => sub_id, // Return static reference.
            FixTag::MsgSeqNum(seq_num) => seq_num, // Use the abstracted function.
            FixTag::SenderLocationID(location_id) => location_id, // Return static reference.
            FixTag::PossDupFlag(f) => f.value(),
            FixTag::OrigSendingTime(orig_time) => orig_time, // Return static reference.
            FixTag::SendingTime(time) => time, // Return static reference.
        }
    }
}

// Add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_tags() {
        let begin_string_tag = FixTag::BeginString(BeginString::Fix4_2);
        assert_eq!(begin_string_tag.tag_id(), 8);
        assert_eq!(begin_string_tag.field_name(), "BeginString");
        assert_eq!(begin_string_tag.value(), "FIX.4.2");

        let poss_dup_tag = FixTag::PossDupFlag(PossDupFlag::Yes);
        assert_eq!(poss_dup_tag.tag_id(), 43);
        assert_eq!(poss_dup_tag.field_name(), "PossDupFlag");
        assert_eq!(poss_dup_tag.value(), "Y");

        let sender_comp_id_tag = FixTag::SenderCompID(CompID::new("Sender123"));
        assert_eq!(sender_comp_id_tag.tag_id(), 49);
        assert_eq!(sender_comp_id_tag.field_name(), "SenderCompID");
        assert_eq!(sender_comp_id_tag.value(), "Sender123");

        let target_comp_id_tag = FixTag::TargetCompID(CompID::new("Target123"));
        assert_eq!(target_comp_id_tag.tag_id(), 56);
        assert_eq!(target_comp_id_tag.field_name(), "TargetCompID");
        assert_eq!(target_comp_id_tag.value(), "Target123");

        let msg_seq_num_tag = FixTag::MsgSeqNum("0");
        assert_eq!(msg_seq_num_tag.tag_id(), 34);
        assert_eq!(msg_seq_num_tag.field_name(), "MsgSeqNum");
        assert_eq!(msg_seq_num_tag.value(), "0");
    }
}
