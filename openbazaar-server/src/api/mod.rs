use std::collections::{HashMap, VecDeque};

use crate::network::Client;
use crate::openbazaar::open_bazaar_api_server::OpenBazaarApi;
use crate::openbazaar::{MessageLocationResponse, NodeLocationRequest, NodeLocationResponse};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct OpenBazaarApiService {
    client: Client,
}

impl OpenBazaarApiService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl OpenBazaarApi for OpenBazaarApiService {
    async fn look_up(
        &self,
        request: Request<NodeLocationRequest>,
    ) -> Result<Response<NodeLocationResponse>, Status> {
        let address = request.into_inner().address;
        let peer_id = match self.client.get_closest_peers(address).await {
            Ok(d) => d,
            Err(e) => return Err(Status::internal(e.to_string())),
        };
        let nodedata = match self.client.get_clear_address(peer_id).await {
            Ok(d) => d,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

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
}
