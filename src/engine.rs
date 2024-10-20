use crate::message::FixMessage;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tracing::*;
use crate::clock::Clock;
use crate::tag::SOH;

#[derive(Debug, Clone)]
pub enum FixEngineMode {
    Initiator,
    Acceptor
}

pub struct FixEngine {
    clock: Arc<dyn Clock>,
    engine_mode: FixEngineMode, // No 'static lifetime constraint
    is_running: Arc<AtomicBool>, // Use AtomicBool instead of Arc<Mutex<bool>>
    send_thread: Option<thread::JoinHandle<()>>,
    receive_thread: Option<thread::JoinHandle<()>>,
}

impl FixEngine {
    pub fn new(clock: Arc<dyn Clock>, engine_mode: FixEngineMode) -> FixEngine {
        FixEngine {
            clock,
            engine_mode,
            is_running: Arc::new(AtomicBool::new(true)), // Use AtomicBool
            send_thread: None,
            receive_thread: None,
        }
    }

    pub fn start(&mut self, mut stream: TcpStream, outgoing_receiver: Receiver<FixMessage>, incoming_sender: Sender<FixMessage>) -> std::io::Result<()> {

        // Receiver thread (reads from TCP stream)
        let clock = Arc::clone(&self.clock);
        let mode = self.engine_mode.clone();
        let stream_clone = stream.try_clone()?;
        let is_running_receive_thread = Arc::clone(&self.is_running);

        self.receive_thread = Some(thread::spawn(move || {
            info!("{:?}: Ready to receive messages.", mode);
            let mut buffer = vec![];
            let mut stream_reader = stream_clone;
            if let Err(e) = stream_reader.set_read_timeout(Some(Duration::from_secs(1))) {
                error!("{:?}: Error setting read timeout: {:?}", mode, e);
                return;
            }

            while is_running_receive_thread.load(Ordering::Relaxed) {
                let mut tmp_buf = [0; 512];
                match stream_reader.read(&mut tmp_buf) {
                    Ok(size) => {
                        if size > 0 {
                            buffer.extend_from_slice(&tmp_buf[..size]);

                            if let Some((message_str, remaining)) = extract_message(&buffer) {
                                if let Ok(fix_message) = FixMessage::decode(&message_str) {
                                    info!("{:?}: Received message {:?}", mode, fix_message);
                                    if let Err(e) = incoming_sender.send(fix_message) {
                                        error!("{:?}: Error sending message: {:?}", mode, e);
                                    }
                                }
                                buffer = remaining;
                            }
                        }
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        if !is_running_receive_thread.load(Ordering::Relaxed) {
                            info!("{:?}: Shutdown signal received, exiting receive thread.", mode);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("{:?}: Error reading from stream: {:?}", mode, e);
                        break;
                    }
                }
            }
        }));

        // Sender thread (writes to TCP stream)
        let mode = self.engine_mode.clone();
        let is_running_send_thread = Arc::clone(&self.is_running);

        self.send_thread = Some(thread::spawn(move || {
            info!("{:?}: Ready to send messages.", mode);
            while is_running_send_thread.load(Ordering::Relaxed) {
                if let Ok(mut message) = outgoing_receiver.recv_timeout(Duration::from_secs(1)) {
                    info!("{:?}: Sending message {:?}", mode, message);
                    let message_str = message.encode(&clock);
                    if let Err(e) = stream.write_all(message_str.as_bytes()) {
                        error!("{:?}: Error writing to stream: {:?}", mode, e);
                    }
                }

                if !is_running_send_thread.load(Ordering::Relaxed) {
                    info!("{:?}: Shutdown signal received, exiting send thread.", mode);
                    break;
                }
            }
        }));

        Ok(())
    }

    pub fn shutdown(&mut self) {
        info!("{:?}: Shutting down.", self.engine_mode);
        self.is_running.store(false, Ordering::Relaxed);

        if let Some(tx_thread) = self.send_thread.take() {
            if let Err(e) = tx_thread.join() {
                error!("{:?}: Error joining tx_thread: {:?}", self.engine_mode, e);
            }
        }

        if let Some(rx_thread) = self.receive_thread.take() {
            if let Err(e) = rx_thread.join() {
                error!("{:?}: Error joining rx_thread: {:?}", self.engine_mode, e);
            }
        }

        info!("{:?}: Fully shut down.", self.engine_mode);
    }
}

// Extracts a complete FIX message from the buffer and returns the remaining unprocessed data.
fn extract_message(buffer: &[u8]) -> Option<(String, Vec<u8>)> {
    let message_str = String::from_utf8_lossy(buffer).to_string();

    if let Some(checksum_pos) = message_str.find("10=") {

        if let Some(end_pos) = message_str[checksum_pos..].find(SOH) {
            let full_message = &message_str[..checksum_pos + end_pos + 1]; // Include '10=xxx' and SOH
            let remaining_data = buffer[(checksum_pos + end_pos + 1)..].to_vec(); // Remaining bytes
            return Some((full_message.to_string(), remaining_data));
        }
    }
    None
}
