use crate::OpenBazaarApi::open_bazaar_api_client::OpenBazaarApiClient;
use crate::{NodeAddressType, OpenBazaarApi::NodeLocationRequest};
use tokio::runtime::Runtime;
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

    // Get a response from the client lookup

    // Return addr and addr_type

    Ok((NodeAddressType::Clear, String::from("test")))
}
