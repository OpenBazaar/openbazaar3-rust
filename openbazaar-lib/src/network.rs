use crate::open_bazaar_api::open_bazaar_api_client::OpenBazaarApiClient;
use crate::open_bazaar_api::{NodeAddressType, NodeLocationRequest};
use tokio::runtime::Runtime;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("Error in tonic transport layer")]
    TonicError {
        #[from]
        source: tonic::transport::Error,
    },
    #[error("Server threw error with responding status => {0:?}")]
    ServerResponseErr(Status),
}

impl From<i32> for NodeAddressType {
    fn from(number: i32) -> Self {
        match number {
            1 => Self::Onion,
            2 => Self::Clear,
            3 => Self::Ipv4,
            4 => Self::Ipv6,
            _ => {
                panic!("This shouldn't have happened")
            }
        }
    }
}

pub fn get_server_for_address(
    rt: &Runtime,
    server_address: String,
    message_address: &[u8],
) -> Result<(NodeAddressType, String), NetError> {
    let mut client = match rt.block_on(OpenBazaarApiClient::connect(server_address)) {
        Ok(d) => d,
        Err(e) => return Err(NetError::TonicError { source: e }),
    };

    // Craft request object
    let request = Request::new(NodeLocationRequest {
        address: message_address.to_vec(),
    });

    println!("Request: {:?}", request);

    // Get a response from the client lookup
    let response = match rt.block_on(client.look_up(request)) {
        Ok(d) => d,
        Err(e) => return Err(NetError::ServerResponseErr(e)),
    };

    // Return addr and addr_type
    let response = response.into_inner();
    let address_type = NodeAddressType::from(response.address_type);
    let address = response.address;

    Ok((address_type, address))
}
