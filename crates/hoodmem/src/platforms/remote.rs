use std::{
    io::{Read, Write},
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{MemoryRegion, Process};
use anyhow::Result;
use interprocess::local_socket::{
    prelude::*, traits::Stream, GenericNamespaced, ListenerOptions, Name, ToNsName,
};

use bincode::{decode_from_slice, encode_into_slice, Decode, Encode};

#[derive(Debug, Clone)]
pub struct RemoteProcess {
    socket: Arc<Mutex<LocalSocketStream>>,
}

#[derive(Debug, Encode, Decode)]
pub enum RPCMessage {
    ReadMemory {
        address: usize,
        bytes_to_read: usize,
    },
    ReadMemoryResult(Vec<u8>),
    GetWritableRegions,
    GetWritableRegionsResult(Vec<MemoryRegion>),
    Error(String),
}

impl Process for RemoteProcess {
    fn read_memory_bytes(&self, address: usize, bytes_to_read: usize) -> Result<Vec<u8>> {
        if let Ok(mut sock) = self.socket.lock() {
            send_message(
                &mut *sock,
                RPCMessage::ReadMemory {
                    address,
                    bytes_to_read,
                },
            )?;
            let result = recv_message(&mut *sock)?;
            match result {
                RPCMessage::ReadMemoryResult(items) => Ok(items),
                RPCMessage::Error(err) => anyhow::bail!(err),
                _ => anyhow::bail!("Unexpected response from remote process..."),
            }
        } else {
            anyhow::bail!("Failed to read memory");
        }
    }

    fn get_writable_regions(&self) -> Vec<MemoryRegion> {
        if let Ok(mut sock) = self.socket.lock() {
            if let Err(err) = send_message(&mut *sock, RPCMessage::GetWritableRegions) {
                eprintln!("Error requesting writable regions from remote process: {err}");
                return vec![];
            }
            let result = recv_message(&mut *sock);
            match result {
                Ok(RPCMessage::GetWritableRegionsResult(regions)) => regions,
                _ => {
                    eprintln!("Unexpected response from remote process...");
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }
}

impl RemoteProcess {
    pub fn connect(address: &str) -> Result<Self> {
        let ns_name: Name = address.to_ns_name::<GenericNamespaced>()?;
        Ok(Self {
            socket: Arc::new(Mutex::new(LocalSocketStream::connect(ns_name)?)),
        })
    }
}

fn send_message(sock: &mut impl Stream, msg: RPCMessage) -> Result<()> {
    let mut msg_buf: Vec<u8> = Vec::with_capacity(size_of::<RPCMessage>());
    unsafe {
        msg_buf.set_len(size_of::<RPCMessage>());
    }
    encode_into_slice(msg, &mut msg_buf, bincode::config::standard())?;
    sock.write_all(&msg_buf)?;
    Ok(())
}

fn recv_message(sock: &mut impl Stream) -> Result<RPCMessage> {
    let mut msg_buf: Vec<u8> = Vec::with_capacity(size_of::<RPCMessage>());
    unsafe {
        msg_buf.set_len(size_of::<RPCMessage>());
    }
    sock.read_exact(&mut msg_buf)?;
    let (msg, _) = decode_from_slice::<RPCMessage, _>(&msg_buf, bincode::config::standard())?;
    Ok(msg)
}

pub struct RemoteProcessServer {
    listen_sock: LocalSocketListener,
    local_process: Arc<dyn crate::Process>,
}

impl RemoteProcessServer {
    pub fn listen(address: &str, pid: u32) -> Result<Self> {
        let ns_name: Name = address.to_ns_name::<GenericNamespaced>()?;
        let listen_opts = ListenerOptions::new().name(ns_name);

        let local_process: Arc<dyn Process> = crate::attach_external(pid)?;

        println!("Attached to {pid} and listening on {address}...");

        Ok(RemoteProcessServer {
            listen_sock: listen_opts.create_sync()?,
            local_process,
        })
    }

    pub fn run(&self) {
        for conn in self.listen_sock.incoming() {
            if let Ok(mut conn) = conn {
                println!("Got new connection {conn:?}...");
                loop {
                    let msg = recv_message(&mut conn);
                    match msg {
                        Ok(msg) => {
                            self.handle_message(msg, &mut conn);
                        }
                        Err(err) => {
                            eprintln!("Error reading message from {conn:?}: {err}");
                            break;
                        }
                    }
                }
                println!("Terminating connection {conn:?}...");
            }
        }
    }

    fn handle_message(&self, msg: RPCMessage, stream: &mut impl Stream) {
        println!("Got message: {msg:?}");

        match msg {
            RPCMessage::ReadMemory {
                address,
                bytes_to_read,
            } => {
                let result = self.local_process.read_memory_bytes(address, bytes_to_read);
                let _ = match result {
                    Ok(result) => send_message(stream, RPCMessage::ReadMemoryResult(result)),
                    Err(err) => send_message(stream, RPCMessage::Error(err.to_string())),
                };
            }
            RPCMessage::GetWritableRegions => {
                let result = self.local_process.get_writable_regions();
                let _ = send_message(stream, RPCMessage::GetWritableRegionsResult(result));
            }
            _ => {
                eprintln!("Got unknown message from client")
            }
        }
    }
}
