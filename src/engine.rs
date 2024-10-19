use crate::message::{Clock, FixMessage};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::*;
use crate::protocol::SOH;

pub struct FixEngine {
    engine_mode: &'static str,
    is_running: Arc<Mutex<bool>>,
    tx_thread: Option<thread::JoinHandle<()>>,
    rx_thread: Option<thread::JoinHandle<()>>,
    clock: Arc<dyn Clock>,
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

        // Receiver thread (reads from TCP stream)
        let stream_clone = stream.try_clone().unwrap();
        let is_running_rx = Arc::clone(&is_running);
        self.rx_thread = Some(thread::spawn(move || {
            info!("{0}: Ready to receive messages.", mode);
            let mut buffer = vec![];
            let mut stream_reader = stream_clone;
            stream_reader.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

            while *is_running_rx.lock().unwrap() {
                let mut tmp_buf = [0; 512]; // Temporary buffer for incoming chunks
                match stream_reader.read(&mut tmp_buf) {
                    Ok(size) => {
                        if size > 0 {
                            buffer.extend_from_slice(&tmp_buf[..size]); // Append the chunk to the buffer

                            if let Some((message_str, remaining)) = extract_message(&buffer) {
                                if let Ok(fix_message) = FixMessage::decode(&message_str, Arc::clone(&clock)) {
                                    info!("{0}: Received message {fix_message:?}.", mode);
                                    incoming_tx.send(fix_message).unwrap();
                                }
                                buffer = remaining; // Preserve remaining unprocessed data for next loop
                            }
                        }
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        if !*is_running_rx.lock().unwrap() {
                            info!("{0}: Shutdown signal received, exiting receive thread.", mode);
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }));

        // Sender thread (writes to TCP stream)
        let is_running_tx = Arc::clone(&is_running);
        self.tx_thread = Some(thread::spawn(move || {
            info!("{0}: Ready to send messages.", mode);
            while *is_running_tx.lock().unwrap() {
                if let Ok(mut message) = outgoing_rx.recv_timeout(Duration::from_secs(1)) {
                    info!("{0}: Sending message {message:?}.", mode);
                    let message_str = message.encode();
                    stream.write_all(message_str.as_bytes()).unwrap();
                }
                if !*is_running_tx.lock().unwrap() {
                    info!("{0}: Shutdown signal received, exiting send thread.", mode);
                    break;
                }
            }
        }));
    }

    pub fn shutdown(&mut self) {
        info!("{0}: Shutting down.", self.engine_mode);

        {
            let mut is_running = self.is_running.lock().unwrap();
            *is_running = false;
        }

        // If the sender/receiver threads are using the stream, flush and close the stream
        if let Some(tx_thread) = self.tx_thread.take() {
            if let Err(e) = tx_thread.join() {
                error!("Error joining tx_thread: {:?}", e);
            }
        }

        if let Some(rx_thread) = self.rx_thread.take() {
            if let Err(e) = rx_thread.join() {
                error!("Error joining rx_thread: {:?}", e);
            }
        }

        info!("{0}: Fully shut down.", self.engine_mode);
    }
}

// Extracts a complete FIX message from the buffer and returns the remaining unprocessed data.
fn extract_message(buffer: &[u8]) -> Option<(String, Vec<u8>)> {
    let message_str = String::from_utf8_lossy(buffer).to_string();

    if let Some(checksum_pos) = message_str.find("10=") {

        if let Some(end_pos) = message_str[checksum_pos..].find(SOH) {
            let full_message = &message_str[..checksum_pos + end_pos +1]; // Include '10=xxx' and SOH
            let remaining_data = buffer[(checksum_pos + end_pos +1)..].to_vec(); // Remaining bytes
            return Some((full_message.to_string(), remaining_data));
        }
    }
    None
}