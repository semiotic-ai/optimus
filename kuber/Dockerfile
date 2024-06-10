FROM debian:bullseye

RUN apt-get update && \
            apt-get install -y make wget&& \
            rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ./Makefile ./Makefile

RUN make install

# setup + sink
ENTRYPOINT ["make"]
CMD ["execute"]

