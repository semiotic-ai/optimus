STREAMINGFAST_GIT_REPO ?= https://github.com/streamingfast
SUBSTREAMS_VERSION ?= v1.1.20
SQLSINK_VERSION ?= v3.0.5
PLATFORM ?= $(shell uname -s | tr A-Z a-z)
ARCH ?= $(shell arch | sed s/aarch64/arm64/ | sed s/amd64/x86_64/)

SUBSTREAMS_SETUP ?= substreams_$(PLATFORM)_${ARCH}.tar.gz 

#SQLSINK_SETUP ?= substreams-sink-sql_linux_${ARCH}.tar.gz
SQLSINK_SETUP ?= substreams-sink-postgres_$(PLATFORM)_${ARCH}.tar.gz

SUBSTREAMS_DOWNLOAD ?= ${STREAMINGFAST_GIT_REPO}/substreams/releases/download/${SUBSTREAMS_VERSION}/${SUBSTREAMS_SETUP}
#SQLSINK_DOWNLOAD ?= ${STREAMINGFAST_GIT_REPO}/substreams-sink-sql/releases/download/${SQLSINK_VERSION}/${SQLSINK_SETUP}
SQLSINK_DOWNLOAD ?= https://github.com/semiotic-ai/substreams-sink-postgres/releases/download/v2.0.0/${SQLSINK_SETUP}

UNDO_BUFFER_SIZE ?= 15
FLUSH_INTERVAL ?= 15


install:
	@if [ ! -f ./bin/substreams ]; then \
		mkdir -p bin && \
		cd bin && \
		wget ${SUBSTREAMS_DOWNLOAD} && \
		tar -xvf ${SUBSTREAMS_SETUP}; \
    fi
	@if [ ! -f ./bin/substreams-sink-sql ]; then \
		mkdir -p bin && \
		cd bin && \
		wget ${SQLSINK_DOWNLOAD} && \
		tar -xvf ${SQLSINK_SETUP}; \
    fi

codegen:
	./bin/substreams protogen ./price/substreams.yaml --exclude-paths="sf/substreams,google"

build:
	cargo build --target wasm32-unknown-unknown --release --package optimus-prices
	./bin/substreams pack ./prices/substreams.yaml -o ./prices/substreams.spkg  

setup: 
	./bin/substreams-sink-sql setup "${DATABASE_URL}" "./tx/substreams.spkg"

sink:
	./bin/substreams-sink-sql run $(subst /default,/${TARGET_DATABASE},"${DATABASE_URL}") "./tx/substreams.spkg" --on-module-hash-mistmatch warn --undo-buffer-size ${UNDO_BUFFER_SIZE} --flush-interval ${FLUSH_INTERVAL}

execute:
	until make setup; do sleep 1; done && make sink