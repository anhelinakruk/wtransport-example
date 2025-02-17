FROM rust:1-alpine
RUN apk add --no-cache build-base 

WORKDIR /app

COPY . .
COPY ./certificate.pem /app/certs/certificate.pem
COPY ./certificate.key /app/certs/certificate.key

RUN cargo build --release

