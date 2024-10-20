use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc::{channel, Receiver, Sender}, Arc};
use crate::engine::{FixEngine, FixEngineMode};
use crate::message::FixMessage;
use tracing::{error, info};
use crate::clock::{Clock, RealClock};

pub struct FixEngineFactory;

impl FixEngineFactory {
    pub fn create_initiator(address: &str) -> (FixEngine, Sender<FixMessage>, Receiver<FixMessage>) {
        info!("Creating Initiator.");
        let stream = match TcpStream::connect(address) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to acceptor: {:?}", e);
                panic!("Connection failed");
            }
        };
        info!("Initiator connected to acceptor at {}", address);

        let (outgoing_sender, outgoing_receiver) = channel(); // Send Fix Messages
        let (incoming_sender, incoming_receiver) = channel(); // Receive Fix Messages

        let clock: Arc<dyn Clock> = Arc::new(RealClock);
        let mut engine = FixEngine::new(clock, &FixEngineMode::Initiator);
        engine.start(stream, outgoing_receiver, incoming_sender);
        (engine, outgoing_sender, incoming_receiver)
    }

    pub fn create_acceptor(address: &str) -> (FixEngine, Sender<FixMessage>, Receiver<FixMessage>) {
        info!("Creating Acceptor.");
        let listener = match TcpListener::bind(address) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind to address {}: {:?}", address, e);
                panic!("Acceptor bind failed");
            }
        };
        info!("Acceptor listening on {}", address);

        let (outgoing_sender, outgoing_receiver) = channel(); // Send Fix Messages
        let (incoming_sender, incoming_receiver) = channel(); // Receive Fix Messages

        let stream = listener.accept().unwrap().0;

        let clock: Arc<dyn Clock> = Arc::new(RealClock);
        let mut engine = FixEngine::new(clock, &FixEngineMode::Acceptor);
        engine.start(stream, outgoing_receiver, incoming_sender);
        (engine, outgoing_sender, incoming_receiver)
    }
}
