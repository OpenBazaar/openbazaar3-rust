# OpenBazaar 3.0 - Rust Implementation

This mono repository is for OpenBazaar (tentatively OpenBazaar3.0)

## openbazaar-server

This is the base openbazaar server daemon. It will connect to the p2p network, setup the Bitcoin wallet, and gRPC server.

Example commands:
```
cargo run -- start --user <username>
cargo run -- start --user <username> --libp2p-port 4002 --libp2p-hostname 0.0.0.0 --grpc-server 0.0.0.0:8011 --api-server-port 8081
PEER=/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWNo68YnQn5To4LjXHVMkJbuaEFiX2Toa3HwTrnKWSC42R cargo run -- start --user <username> --libp2p-port 4002 --libp2p-hostname 0.0.0.0 --grpc-server 0.0.0.0:8011 --api-server-port 8081 
```

## openbazaar-web

This is the React.js web application for interacting with OpenBazaar.

## openbazaar-lib

A Rust library for building Rust applications using the gRPC server for openbazaar-server

## openbazaar-cli 

A simple command-line interface for testing out OpenBazaar capabilities