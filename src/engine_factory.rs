use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc::{channel, Sender, Receiver}};
use crate::fix_engine::FixEngine;
use crate::fix_message::{FixMessage, RealClock, Clock};
use tracing::{error, info};

pub struct FixEngineFactory;

const INITIATOR: &'static str = "Initiator";
const ACCEPTOR: &'static str = "Acceptor";

impl FixEngineFactory {
    pub fn create_initiator(address: &str) -> (FixEngine, Sender<FixMessage>, Receiver<FixMessage>) {
        info!("Creating Initiator.");
        // Connect to the acceptor
        let stream = match TcpStream::connect(address) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to acceptor: {:?}", e);
                panic!("Connection failed");
            }
        };
        info!("Initiator connected to acceptor at {}", address);

        let (tx, rx) = channel();
        let (incoming_tx, incoming_rx) = channel();

        let clock: Arc<dyn Clock> = Arc::new(RealClock); // Provide the clock
        let mut engine = FixEngine::new(clock, INITIATOR); // Pass the clock into FixEngine
        engine.start(stream, rx, incoming_tx);
        (engine, tx, incoming_rx)
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

        let (tx, rx) = channel();
        let (incoming_tx, incoming_rx) = channel();

        let stream = listener.accept().unwrap().0;

        let clock: Arc<dyn Clock> = Arc::new(RealClock); // Provide the clock
        let mut engine = FixEngine::new(clock, ACCEPTOR); // Pass the clock into FixEngine
        engine.start(stream, rx, incoming_tx);
        (engine, tx, incoming_rx)
    }
}
