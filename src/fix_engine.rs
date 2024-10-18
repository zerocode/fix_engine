use crate::fix_message::{Clock, FixMessage, SOH};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::info;

pub struct FixEngine {
    engine_mode: &'static str,
    is_running: Arc<Mutex<bool>>,
    tx_thread: Option<thread::JoinHandle<()>>,
    rx_thread: Option<thread::JoinHandle<()>>,
    clock: Arc<dyn Clock>, // Inject clock for time management
}

impl FixEngine {
    pub fn new(clock: Arc<dyn Clock>, mode: &'static str) -> FixEngine {
        FixEngine {
            engine_mode: mode,
            is_running: Arc::new(Mutex::new(true)),
            tx_thread: None,
            rx_thread: None,
            clock,
        }
    }

    pub fn start(&mut self, mut stream: TcpStream, outgoing_rx: Receiver<FixMessage>, incoming_tx: Sender<FixMessage>) {
        let is_running = Arc::clone(&self.is_running);
        let clock = Arc::clone(&self.clock);
        let mode = self.engine_mode.clone();

        info!("{0}: Starting.", mode);

        // Receiver thread (reads from TCP stream)
        let stream_clone = stream.try_clone().unwrap();
        let is_running_rx = Arc::clone(&is_running);
        self.rx_thread = Some(thread::spawn(move || {
            info!("{0}: Running receive thread.", mode);
            let mut buffer = vec![];
            let mut stream_reader = stream_clone;

            while *is_running_rx.lock().unwrap() {
                let mut tmp_buf = [0; 512]; // Temporary buffer for incoming chunks
                match stream_reader.read(&mut tmp_buf) {
                    Ok(size) => {
                        if size > 0 {
                            buffer.extend_from_slice(&tmp_buf[..size]); // Append the chunk to the buffer

                            // Try to find a complete message
                            if let Some((message_str, remaining)) = extract_complete_message(&buffer) {
                                if let Ok(fix_message) = FixMessage::decode(&message_str, Arc::clone(&clock)) {
                                    info!("{0}: Received message {fix_message:?}.", mode);
                                    incoming_tx.send(fix_message).unwrap();
                                }
                                buffer = remaining; // Preserve remaining unprocessed data for next loop
                            }
                        }
                    },
                    Err(_) => break,
                }
            }
        }));

        // Sender thread (writes to TCP stream)
        let is_running_tx = Arc::clone(&is_running);
        self.tx_thread = Some(thread::spawn(move || {
            info!("{0}: Running send thread.", mode);
            while *is_running_tx.lock().unwrap() {
                if let Ok(mut message) = outgoing_rx.recv_timeout(Duration::from_secs(1)) {
                    info!("{0}: Sending message {message:?}.", mode);
                    let message_str = message.encode();
                    stream.write_all(message_str.as_bytes()).unwrap();
                }
            }
        }));
    }

    pub fn shutdown(&mut self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;

        if let Some(tx_thread) = self.tx_thread.take() {
            tx_thread.join().unwrap();
        }
        if let Some(rx_thread) = self.rx_thread.take() {
            rx_thread.join().unwrap();
        }
    }
}

// Extracts a complete FIX message from the buffer and returns the remaining unprocessed data.
fn extract_complete_message(buffer: &[u8]) -> Option<(String, Vec<u8>)> {
    let message_str = String::from_utf8_lossy(buffer).to_string();

    // Check if the message contains a complete "10=xxx" (checksum) field followed by SOH
    if let Some(checksum_pos) = message_str.find("10=") {
        // Look for the SOH character after the checksum value
        if let Some(end_pos) = message_str[checksum_pos..].find(SOH) {
            // Extract the complete message
            let full_message = &message_str[..checksum_pos + end_pos +1]; // Include '10=xxx' and SOH
            let remaining_data = buffer[(checksum_pos + end_pos +1)..].to_vec(); // Remaining bytes
            return Some((full_message.to_string(), remaining_data));
        }
    }
    None // If no complete message is found, return None
}