# optimus-events

Generate substreams packages based on ABI. All the code is auto-generated including:

- schema.sql (for Clickhouse)
- substreams.yaml
- substreams.spkg

## Prerequisites

- substreams
- protogen
- make
- rust
- wasm32-unknown-unknown target


## Creating your substreams.spkg

### Configuration

Add the ABI files to `abi/` and update the configuration file at `substream_config.json`. For example: 
```
{
    "name": "erc20",
    "version": "v1.0.0",
    "network": "mainnet",
    "initial_block": 1,
    "database": "erc20",
    "contracts":[
        {
            "name":"erc20",
            "table_prefix":"evt_",
            "abi_file":"./abi/erc20.json"
        }
    ]
}
```

Please take a look at [configs](configs/) to see more examples.


### Generating the output

After everything is configured, just run:
```
make compile
```

It will build the wasm file and use substreams to compile.

After that, you can take your files in: `output/`.

