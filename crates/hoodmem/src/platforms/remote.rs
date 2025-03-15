use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use crate::{MemoryRegion, Process};
use anyhow::Result;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use serde::{Deserialize, Serialize};

struct IPCConnection {
    server_name: String,
    sender: ipc_channel::ipc::IpcSender<RPCMessage>,
    receiver: ipc_channel::ipc::IpcReceiver<RPCMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RPCMessage {
    InitConnection {
        token: String,
    },
    AckConnection,
    ReadMemory {
        address: usize,
        bytes_to_read: usize,
    },
    ReadMemoryResult(Vec<u8>),
    GetWritableRegions,
    GetWritableRegionsResult(Vec<MemoryRegion>),
    Error(String),
}

pub struct RemoteProcess {
    conn: Arc<Mutex<IPCConnection>>,
}

impl RemoteProcess {
    pub fn connect(addr: &str) -> Result<Self> {
        Ok(Self {
            conn: Arc::new(Mutex::new(IPCConnection::connect(addr)?)),
        })
    }
}

impl Process for RemoteProcess {
    fn read_memory_bytes(&self, address: usize, bytes_to_read: usize) -> Result<Vec<u8>> {
        if let Ok(conn) = self.conn.lock() {
            conn.sender.send(RPCMessage::ReadMemory {
                address,
                bytes_to_read,
            })?;
            let result = conn.receiver.recv()?;
            match result {
                RPCMessage::ReadMemoryResult(items) => Ok(items),
                RPCMessage::Error(err) => anyhow::bail!(err),
                _ => anyhow::bail!("Unexpected response from remote process..."),
            }
        } else {
            anyhow::bail!("Couldn't acquire stream lock");
        }
    }

    fn get_writable_regions(&self) -> Vec<MemoryRegion> {
        if let Ok(conn) = self.conn.lock() {
            if let Err(err) = conn.sender.send(RPCMessage::GetWritableRegions) {
                eprintln!("Error requesting writable regions from remote process: {err}");
                return vec![];
            }
            let result = conn.receiver.recv();
            println!("Found writable regions response: {result:#?}");
            match result {
                Ok(RPCMessage::GetWritableRegionsResult(regions)) => regions,
                _ => {
                    eprintln!("Unexpected response from remote process...");
                    vec![]
                }
            }
        } else {
            eprintln!("Couldn't acquire stream lock");
            vec![]
        }
    }
}

impl IPCConnection {
    /// Used to start listening for a connection (as a server)
    pub fn listen() -> Result<Self> {
        let (rx_server, rx_server_name) = IpcOneShotServer::<RPCMessage>::new()?;

        println!("Server listening for new connection with name {rx_server_name}...");

        // Receive token from a client to connect back
        let (rx, msg) = rx_server.accept()?;

        if let RPCMessage::InitConnection { token } = msg {
            println!(
                "Server received new connection. Connecting back to client with token {token}..."
            );
            let tx = IpcSender::<RPCMessage>::connect(token)?;

            tx.send(RPCMessage::AckConnection);

            Ok(Self {
                server_name: rx_server_name,
                sender: tx,
                receiver: rx,
            })
        } else {
            anyhow::bail!("Server did not receive a InitConnection message from client");
        }
    }

    /// Used to establish bidrectional communication with a listening server (as a client)
    pub fn connect(address: &str) -> Result<Self> {
        let tx = IpcSender::connect(address.into())?;
        let (rx_server, rx_server_name) = IpcOneShotServer::<RPCMessage>::new()?;

        // Init the connection
        tx.send(RPCMessage::InitConnection {
            token: rx_server_name.clone(),
        })?;

        let (rx, _msg) = rx_server.accept()?;

        Ok(Self {
            server_name: rx_server_name,
            sender: tx,
            receiver: rx,
        })
    }
}

pub struct RemoteProcessServer {
    conn: Arc<Mutex<IPCConnection>>,
    local_process: Arc<dyn Process>,
}

impl RemoteProcessServer {
    pub fn listen(pid: u32) -> Result<Self> {
        let conn = IPCConnection::listen()?;
        let local_process: Arc<dyn Process> = crate::attach_external(pid)?;

        Ok(RemoteProcessServer {
            conn: Arc::new(Mutex::new(conn)),
            local_process,
        })
    }

    pub fn run(&self) {
        loop {
            if let Ok(conn) = self.conn.lock() {
                let msg = conn.receiver.recv();
                match msg {
                    Ok(msg) => {
                        self.handle_message(msg);
                    }
                    Err(err) => {
                        eprintln!("Error reading message: {err}. Aborting connection");
                        break;
                    }
                }
            }
        }
    }

    fn handle_message(&self, msg: RPCMessage) {
        println!("Got message: {msg:?}");

        match msg {
            RPCMessage::ReadMemory {
                address,
                bytes_to_read,
            } => {
                let result = self.local_process.read_memory_bytes(address, bytes_to_read);

                if let Ok(conn) = self.conn.lock() {
                    match result {
                        Ok(result) => conn.sender.send(RPCMessage::ReadMemoryResult(result)),
                        Err(err) => conn.sender.send(RPCMessage::Error(err.to_string())),
                    };
                } else {
                    eprintln!("Failed to acquire connection lock...");
                }
            }
            RPCMessage::GetWritableRegions => {
                let result = self.local_process.get_writable_regions();
                if let Ok(conn) = self.conn.lock() {
                    let msg_result = conn
                        .sender
                        .send(RPCMessage::GetWritableRegionsResult(result));
                    if let Err(err) = msg_result {
                        eprintln!("Error sending GetWritableRegionResult msg: {err}");
                    }
                } else {
                    eprintln!("Failed to acquire connection lock...");
                }
            }
            _ => {
                eprintln!("Got unknown message from client")
            }
        }
    }
}
