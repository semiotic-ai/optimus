STREAMINGFAST_GIT_REPO ?= https://github.com/streamingfast

SQLSINK_VERSION ?= v3.0.5
ARCH ?= $(shell arch | sed s/aarch64/arm64/ | sed s/amd64/x86_64/)

#SQLSINK_SETUP ?= substreams-sink-sql_linux_${ARCH}.tar.gz
SQLSINK_SETUP ?= substreams-sink-postgres_linux_x86_64.tar.gz

#SQLSINK_DOWNLOAD ?= ${STREAMINGFAST_GIT_REPO}/substreams-sink-sql/releases/download/${SQLSINK_VERSION}/${SQLSINK_SETUP}
SQLSINK_DOWNLOAD ?= https://github.com/semiotic-ai/substreams-sink-postgres/releases/download/v2.0.0/${SQLSINK_SETUP}

UNDO_BUFFER_SIZE ?= 15
FLUSH_INTERVAL ?= 15

guard-%:
	echo @#$(or ${$*}, $(error $* is not set))

install:
	wget ${SQLSINK_DOWNLOAD} && \
	tar -xvf ${SQLSINK_SETUP}; 

setup: guard-DATABASE_URL guard-SUBSTREAMS_PACKAGE
	./substreams-sink-sql setup "${DATABASE_URL}" "${SUBSTREAMS_PACKAGE}"

sink: guard-DATABASE_URL guard-SUBSTREAMS_PACKAGE guard-TARGET_DATABASE
	./substreams-sink-sql run $(subst /default,/${TARGET_DATABASE},"${DATABASE_URL}") "${SUBSTREAMS_PACKAGE}" --on-module-hash-mistmatch warn --undo-buffer-size ${UNDO_BUFFER_SIZE} --flush-interval ${FLUSH_INTERVAL}

execute:
	until make setup; do sleep 1; done && make sink
