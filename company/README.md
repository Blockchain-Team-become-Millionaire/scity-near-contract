# Scity NFT Land Contract

Scity - the next generation of metaverse.

## Installation

Build contract.

```bash
./build.sh
```

Deploy contract to NEAR testnet.

```bash
near deploy --wasmFile main.wasm  --accountId [your_account_id]
```

## Usage

Senario:

##### 1. Create new contract instance.

```bash
near call $ID new_default_meta '{"owner_id":[owner_id]}' --accountId [your_account_id]
```

##### 2. Open new area.

```bash
near call $ID open_area '{"name": "tokyo", "limit": 12, "price": [yoctoNear], "open_time": [nanoseconds], "close_time": [nanoseconds]}' --accountId [your_account_id]
```

##### 1. Buy land.

```bash
near call $ID buy_land '{"name": [area_name]}' --accountId [your_account_id] --depositYocto [yotoNear]
```

## License

[MIT](https://choosealicense.com/licenses/mit/)

## Author

LocDT <<locdt.developer@gmail.com>>
