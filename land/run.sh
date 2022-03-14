near deploy --wasm-file res/main.wasm --account-id land.dev.scity.testnet --master-account dev.scity.testnet

near call land.dev.scity.testnet new_default_meta '{"owner_id": "dev.scity.testnet"}' --account-id dev.scity.testnet

near call land.dev.scity.testnet open_area '{"name": "toronto", "limit": 900, "price": "100000000000000000000000", "open_time": 1645030800, "close_time": 1650128399}' --account-id dev.scity.testnet