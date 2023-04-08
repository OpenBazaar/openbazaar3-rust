use crate::OpenBazaarApi::NodeAddressType;
use anyhow;
use bincode;
use network::get_server_for_address;
use serde::{Deserialize, Serialize};
use sled::{open, Db};
use std::collections::HashSet;
use std::path::Path;
use tokio::runtime::Runtime;

mod network;

pub(crate) mod OpenBazaarApi {
    tonic::include_proto!("openbazaar_api");
}

pub struct Client {
    db: Db,
    runtime: Runtime,
    openbazaar_nodes: OpenBazaarNodes,
}

impl Client {
    pub fn new<P: AsRef<Path>>(p: P) -> anyhow::Result<Self> {
        let db = open(p)?;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let openbazaar_nodes = OpenBazaarNodes::new(db.clone());

        let client = Self {
            db,
            runtime,
            openbazaar_nodes,
        };

        Ok(client)
    }

    pub fn connect(&mut self, server_address: String) -> anyhow::Result<()> {
        match get_server_for_address(&self.runtime, server_address.clone(), b"") {
            Ok(d) => {
                let hostname = Node::new(NodeAddressType::Clear, server_address);
                let _ = self.openbazaar_nodes.add(hostname);

                Ok(())
            }
            Err(e) => return Err(anyhow::Error::from(e)),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Node {
    pub kind: NodeAddressType,
    pub address: String,
}

impl Node {
    pub fn new(kind: NodeAddressType, address: String) -> Self {
        Self { kind, address }
    }
}

pub struct OpenBazaarNodes {
    db: Db,
    nodes: HashSet<Node>,
}

impl OpenBazaarNodes {
    pub fn new(db: Db) -> Self {
        let nodes = Default::default();

        Self { db, nodes }
    }

    pub fn add(&mut self, node: Node) -> anyhow::Result<()> {
        // Get current nodes from sled db
        let current_nodes: Vec<u8> = self
            .db
            .get(b"nodes")
            .unwrap()
            .expect("Failed to fetch value")
            .to_vec();
        let mut current_nodes: HashSet<Node> = bincode::deserialize(&current_nodes).unwrap();

        current_nodes.insert(node);

        let current_bytes = bincode::serialize(&current_nodes).unwrap();
        self.nodes = current_nodes;
        self.db.insert(b"nodes", current_bytes).unwrap();

        Ok(())
    }
}
