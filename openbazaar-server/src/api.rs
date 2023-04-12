use std::collections::{HashMap, VecDeque};

use crate::db::DB;
use crate::network::Client;
use crate::openbazaar::open_bazaar_rpc_server::OpenBazaarRpc;
use crate::openbazaar::HashType;
use crate::openbazaar::SaveMessageRequest;
use crate::openbazaar::{
    GetMessageRequest, GetMessageResponse, GetProfileRequest, GetProfileResponse,
    MessageLocationResponse, NodeLocationRequest, NodeLocationResponse, SaveMessageResponse,
};
use sha3::{Digest, Sha3_256};
use tonic::{Request, Response, Status};
use tracing::log::trace;
use tracing::{event, instrument, Level};

#[derive(Debug)]
pub struct OpenBazaarRpcService<T: DB> {
    client: Client,
    dbconn: T,
}

impl<T: DB> OpenBazaarRpcService<T> {
    pub fn new(client: Client, dbconn: T) -> Self {
        Self { client, dbconn }
    }
}

#[tonic::async_trait]
impl<T: DB + Sync + Send + 'static> OpenBazaarRpc for OpenBazaarRpcService<T> {
    async fn look_up(
        &self,
        request: Request<NodeLocationRequest>,
    ) -> Result<Response<NodeLocationResponse>, Status> {
        let address = request.into_inner().address;
        println!("Looking up address: {:?}", address);
        let peer_id = match self.client.get_closest_peers(address).await {
            Ok(d) => d,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        // Grab the clear address of the peer
        let nodedata = match self.client.get_clear_address(peer_id).await {
            Ok(d) => d,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        println!("Found peer: {:?}", nodedata);

        let clear_address = nodedata.address;
        let address_type = nodedata.address_type;
        let node_response = NodeLocationResponse {
            address: clear_address,
            address_type: address_type.into(),
        };
        Ok(Response::new(node_response))
    }

    async fn message_look_up(
        &self,
        request: Request<NodeLocationRequest>,
    ) -> Result<Response<MessageLocationResponse>, Status> {
        let address = request.into_inner().address;
        let peer_ids = self.client.get_providers(address.clone()).await;

        let mut peer_queue = VecDeque::with_capacity(peer_ids.len());

        // Transform peer ids queue into a queue of clear address request handles
        for peer_id in peer_ids.clone() {
            let client = self.client.clone();
            let handle = tokio::spawn(async move {
                let nodedata = client.get_clear_address(peer_id).await.unwrap();
                (peer_id, nodedata)
            });
            peer_queue.push_back(handle);
        }

        let mut clear_peers = HashMap::with_capacity(peer_queue.len());

        for found_peer in peer_queue {
            let (peer_id, nodedata) = found_peer.await.unwrap();
            clear_peers.insert(peer_id, nodedata);
        }

        // Iterate through the peers and get their clear addresses and return them as NodeLocationResponses
        let response_addresses = peer_ids
            .iter()
            .map(|peer_id| clear_peers.get(peer_id).unwrap())
            .map(|nodedata| NodeLocationResponse {
                address_type: nodedata.address_type.into(),
                address: nodedata.address.clone(),
            })
            .collect();

        Ok(Response::new(MessageLocationResponse {
            addresses: response_addresses,
        }))
    }

    #[instrument(skip(self, request))]
    async fn save_message(
        &self,
        request: Request<SaveMessageRequest>,
    ) -> Result<Response<SaveMessageResponse>, Status> {
        event!(Level::INFO, "Processing Request");

        let client_clone = self.client.clone();

        let request_data = request.into_inner();
        let addr = request_data.address.clone(); // pull out address to save message in DHT at

        // Spawn a task to propagate the message to the DHT
        let dht = tokio::spawn(async move {
            println!("Propagating {:?}", addr);
            event!(Level::DEBUG, "Propagating to DHT");
            client_clone.start_providing(addr).await;
            event!(Level::DEBUG, "Propagated to DHT");
        });

        let hash = request_data.hash_content();
        event!(Level::DEBUG, "Calculated Hash");
        let reply = SaveMessageResponse { hash: hash };
        event!(Level::DEBUG, "Formulated Response");
        dht.await.unwrap();

        // Save the message to the database
        self.dbconn
            .save_message(&request_data.address, &request_data.content)
            .await
            .expect("Error saving message");

        event!(Level::DEBUG, "Saved to DB");

        Ok(Response::new(reply))
    }

    #[instrument(skip(self, request))]
    async fn get_message(
        &self,
        request: Request<GetMessageRequest>,
    ) -> Result<Response<GetMessageResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        event!(Level::INFO, "Processing Request");

        let request_data = request.into_inner();

        let content = self
            .dbconn
            .get_message(&request_data.address)
            .await
            .expect("Didn't get message")
            .expect("Empty");

        let response = GetMessageResponse {
            address: request_data.address.clone(),
            content: content.clone(),
        };

        Ok(Response::new(response))
    }

    #[instrument(skip(self))]
    async fn get_profile(
        &self,
        _: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileResponse>, Status> {
        event!(Level::INFO, "Processing Profile Request");

        let content = self
            .dbconn
            .get_profile()
            .await
            .expect("Didn't get message")
            .expect("Empty");

        let response = GetProfileResponse {
            id: content.id,
            name: content.name,
            email: content.email,
        };

        Ok(Response::new(response))
    }
}
impl SaveMessageRequest {
    pub fn hash_content(&self) -> Vec<u8> {
        let content = &self.content;
        let mut hasher = Sha3_256::new();
        hasher.update(content);
        let hash = hasher.finalize();
        println!("Hash: {:?}", hash);
        hash.to_vec()
    }
}
