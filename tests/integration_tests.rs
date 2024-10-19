mod fixed_clock;

use crate::fixed_clock::create_fixed_clock;
use fix_engine_2::engine_factory::FixEngineFactory;
use fix_engine_2::message::FixMessage;
use std::thread;
use std::time::Duration;

#[test]
fn test_initiator_acceptor_can_exchange_messages() {
    let address = "127.0.0.1:12345";

    // Start the acceptor in a separate thread
    thread::spawn(move || {
        let (mut engine, sender, receiver) = FixEngineFactory::create_acceptor(address);

        // Receive the message (from initiator)
        let message = receiver.recv().unwrap();
        assert_eq!(message.header.get("35").unwrap(), "A"); // Logon message type

        // Send execution report
        sender.send(create_execution_report()).unwrap();
        engine.shutdown();
    });

    // Give the acceptor time to start listening
    thread::sleep(Duration::from_millis(100));

    // Start the initiator
    let (mut engine, sender, receiver) = FixEngineFactory::create_initiator(address);

    // Create a logon message using the fixed clock
    sender.send(create_logon_message()).unwrap(); // Send logon

    // Receive execution report from acceptor
    let response = receiver.recv().unwrap();
    assert_eq!(response.header.get("35").unwrap(), "8"); // Execution Report message type

    engine.shutdown();
}

fn create_logon_message() -> FixMessage {
    let fixed_clock = create_fixed_clock();
    let mut msg = FixMessage::new(fixed_clock.clone());
    msg.header.insert("8".to_string(), "FIX.4.4".to_string());  // BeginString
    msg.header.insert("35".to_string(), "A".to_string());       // MsgType (Logon)
    msg.header.insert("49".to_string(), "INITIATOR".to_string());  // SenderCompID
    msg.header.insert("56".to_string(), "ACCEPTOR".to_string());  // TargetCompID
    msg.header.insert("34".to_string(), "1".to_string());       // MsgSeqNum
    msg.header.insert("52".to_string(), fixed_clock.now());     // SendingTime
    msg
}

fn create_execution_report() -> FixMessage {
    let fixed_clock = create_fixed_clock();
    let mut msg = FixMessage::new(fixed_clock.clone());
    msg.header.insert("8".to_string(), "FIX.4.4".to_string());
    msg.header.insert("35".to_string(), "8".to_string());  // Execution Report message type
    msg.header.insert("49".to_string(), "ACCEPTOR".to_string());  // SenderCompID
    msg.header.insert("56".to_string(), "INITIATOR".to_string());  // TargetCompID
    msg.header.insert("34".to_string(), "2".to_string());       // MsgSeqNum
    msg.header.insert("52".to_string(), fixed_clock.now());     // SendingTime
    msg
}

#[ctor::ctor]
fn setup() {
    tracing_subscriber::fmt::init();
}
