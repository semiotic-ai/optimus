# optimus-tx
Substream sql sink for ethereum block and transaction data

## Setup

Add wasm target
```sh
rustup target add wasm32-unknown-unknown
```
Install streamingfast package and sink tools
```sh
make install
```

## Build
Build spkg
```sh
make build
```

## Run
Create `.env` file from `.env.example` and run
PS: For initial setup use `default` as database and then update to `ethereum`

```sh
env $(cat .env) make execute
```

## Docker 
Build the docker image
```sh
docker build -t optimus-tx:latest .
```

Run docker image to sync
```sh
docker run --rm --name optimus-tx -it --net=host --env-file=.env optimus-tx:latest
```


