import { OpenBazaarRpcClient } from '../../protobufs/OpenBazaarRpc_grpc_web_pb';

const client = new OpenBazaarRpcClient('http://localhost:8010', {}, '');

export function makeGrpcCall() {

    // Example gRPC calls to the server

    // Find Node
    const request = new proto.openbazaar_rpc.NodeLocationRequest();
    request.setAddress("");

    // Save message to DHT
    const req3 = new proto.openbazaar_rpc.SaveMessageRequest();
    var enc = new TextEncoder();
    req3.setContent(enc.encode("{ test: 'test'}"));
    req3.setAddress("test 55");

    // Get message from DHT
    const req4 = new proto.openbazaar_rpc.GetMessageRequest();
    req4.setAddress("test");

    const findNode = async () => {
        client.messageLookUp(request, {}, (error2, response2) => {
            if (error2) {
                console.error('Error:', error2);
            } else {
                console.log('Response:', response2.toObject());
            }
        });
    }

    const saveAndGet = async () => {
        let response;

        await client.saveMessage(req3, {}, (error3, response3) => {
            if (error3) {
                console.error('Error:', error3);
            } else {
                response = response3.toObject();
                console.log('Response:', response3.toObject());
            }
        });

        await client.getMessage(req4, {}, (error4, response4) => {
            if (error4) {
                console.error('Error:', error4);
            } else {
                console.log('Response:', response4.toObject());

                const dec = new TextDecoder();
                console.log('Response:', dec.decode(response4.getContent_asU8()));
            }
        });
    }

    // findNode();
    // saveAndGet();

    const req = new proto.openbazaar_rpc.GetProfileRequest();
    client.getProfile(req, {}, (error, response) => {
        if (error) {
            console.error('Error:', error);
        } else {
            console.log('Response:', response.toObject());
        }
    });


}