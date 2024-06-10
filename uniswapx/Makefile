ENDPOINT ?= mainnet.eth.streamingfast.io:443
# START_BLOCK ?= 18443569
# START_BLOCK ?= 17777988
START_BLOCK ?= 17782975

STOP_BLOCK ?= 17782980

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: run
run: build
	substreams run -e $(ENDPOINT) substreams.yaml map_fills -s $(START_BLOCK)
	# -t $(STOP_BLOCK)

.PHONY: gui
gui: build
	substreams gui --debug-modules-output logs -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) 
	# -t $(STOP_BLOCK)

.PHONY: protogen
protogen:
	substreams protogen ./substreams.yaml --exclude-paths="google,sf/substreams/sink/database,sf/substreams/rpc,sf/substreams/v1,sf/substreams"

.PHONY: pack
pack: build
	substreams pack substreams.yaml


SUBSTREAMS_PACKAGE_FILE ?= uniswap-x-v0.1.0.spkg
setup:
	substreams-sink-sql setup "${DATABASE_URL}" ${SUBSTREAMS_PACKAGE_FILE}

sink:
	substreams-sink-sql run "${DATABASE_URL}" "${SUBSTREAMS_PACKAGE_FILE}" --on-module-hash-mistmatch warn

execute:
	until make setup; do sleep 1; done && make sink
