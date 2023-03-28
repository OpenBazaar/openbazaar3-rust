FROM rust:latest as builder

RUN apt-get update && apt-get install -y build-essential

WORKDIR /app

# Copy the source code
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/openbazaar3 /usr/local/bin/openbazaar3
# CMD ["/usr/local/bin/openbazaar3", "start"]