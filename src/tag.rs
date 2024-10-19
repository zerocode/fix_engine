pub const SOH: char = '\x01';
const CHECKSUM_TAG: &str = "10";
const REQUIRED_HEADER_FIELDS: [&str; 7] = ["8", "9", "35", "49", "56", "34", "52"];

trait FixField {
    fn tag_id(&self) -> u32;
    fn field_name(&self) -> &'static str;
    fn value(&self) -> String;
}

#[derive(Debug, Clone)]
struct CompID(String);

impl CompID {
    fn new(id: &str) -> Self {
        CompID(id.to_string())
    }
}

impl FixField for CompID {
    fn tag_id(&self) -> u32 {
        49
    }

    fn field_name(&self) -> &'static str {
        "SenderCompID"
    }

    fn value(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
enum PossDupFlag {
    Yes,
    No,
}

impl PossDupFlag {
    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "Y" => Ok(PossDupFlag::Yes),
            "N" => Ok(PossDupFlag::No),
            _ => Err(format!("Invalid PossDupFlag value: {}", value)),
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

    fn value(&self) -> String {
        match self {
            PossDupFlag::Yes => "Y".to_string(),
            PossDupFlag::No => "N".to_string(),
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

    fn value(&self) -> String {
        match self {
            BeginString::Fix4_2 => "FIX.4.2".to_string(),
            BeginString::Fix4_4 => "FIX.4.4".to_string(),
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

    fn value(&self) -> String {
        match self {
            MsgType::Heartbeat => "0".to_string(),
            MsgType::TestRequest => "1".to_string(),
            MsgType::ResendRequest => "2".to_string(),
            MsgType::Reject => "3".to_string(),
            MsgType::SequenceReset => "4".to_string(),
            MsgType::Logout => "5".to_string(),
            MsgType::ExecutionReport => "8".to_string(),
            MsgType::OrderCancelReject => "9".to_string(),
            MsgType::Logon => "A".to_string(),
            MsgType::News => "B".to_string(),
            MsgType::SecurityDefinitionRequest => "c".to_string(),
            MsgType::OrderSingle => "D".to_string(),
            MsgType::SecurityDefinition => "d".to_string(),
            MsgType::SecurityStatusRequest => "e".to_string(),
            MsgType::SecurityStatus => "f".to_string(),
            MsgType::OrderCancelRequest => "F".to_string(),
            MsgType::OrderCancelReplaceRequest => "G".to_string(),
            MsgType::OrderStatusRequest => "H".to_string(),
            MsgType::DontKnowTrade => "Q".to_string(),
            MsgType::QuoteRequest => "R".to_string(),
            MsgType::MarketDataRequest => "V".to_string(),
            MsgType::MarketDataSnapshotFullRefresh => "W".to_string(),
            MsgType::MarketDataIncrementalRefresh => "X".to_string(),
            MsgType::MarketDataRequestReject => "Y".to_string(),
            MsgType::TradeCaptureReportRequest => "AD".to_string(),
            MsgType::TradeCaptureReport => "AE".to_string(),
            MsgType::TradeCaptureReportRequestAck => "AQ".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum FixTag {
    BeginString(BeginString),
    MsgType(MsgType),
    BodyLength(u32),
    SenderCompID(CompID),
    TargetCompID(CompID),
    SenderSubID(String),
    TargetSubID(String),
    OnBehalfOfSubID(String),
    MsgSeqNum(u32),
    SenderLocationID(String),
    PossDupFlag(PossDupFlag),
    OrigSendingTime(String),
    SendingTime(String),
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

    fn value(&self) -> String {
        match self {
            FixTag::BeginString(f) => f.value(),
            FixTag::MsgType(f) => f.value(),
            FixTag::BodyLength(length) => length.to_string(),
            FixTag::SenderCompID(f) => f.value(),
            FixTag::TargetCompID(f) => f.value(),
            FixTag::SenderSubID(sub_id) => sub_id.clone(),
            FixTag::TargetSubID(sub_id) => sub_id.clone(),
            FixTag::OnBehalfOfSubID(sub_id) => sub_id.clone(),
            FixTag::MsgSeqNum(seq_num) => seq_num.to_string(),
            FixTag::SenderLocationID(loc_id) => loc_id.clone(),
            FixTag::PossDupFlag(f) => f.value(),
            FixTag::OrigSendingTime(time) => time.clone(),
            FixTag::SendingTime(time) => time.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_tags() {
        let logon_tag = FixTag::MsgType(MsgType::Logon);
        assert_eq!(logon_tag.tag_id(), 35);
        assert_eq!(logon_tag.field_name(), "MsgType");
        assert_eq!(logon_tag.value(), "A");

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
    }
}
