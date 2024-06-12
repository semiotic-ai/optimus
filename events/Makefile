
build:
	cargo build --target wasm32-unknown-unknown --release

SUBSTREAMS_FILE ?= output/substreams.yaml
compile: build
	substreams pack "${SUBSTREAMS_FILE}" -o output/substreams.spkg  

codegen:
	substreams protogen substreams.yaml --exclude-paths="sf/substreams,google"                                             

run:
	cd output
	substreams run -e mainnet.eth.streamingfast.io:443 "${SUBSTREAMS_FILE}" db_out --start-block 10093000

SUBSTREAMS_PACKAGE_FILE ?= substreams.spkg
setup:
	substreams-sink-sql setup "${DATABASE_URL}" ${SUBSTREAMS_PACKAGE_FILE}

sink:
	substreams-sink-sql run "${DATABASE_URL}" "${SUBSTREAMS_PACKAGE_FILE}" --on-module-hash-mistmatch warn


execute:
	until make setup; do sleep 1; done && make sink
