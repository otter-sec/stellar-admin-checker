# Admin Checker

A command-line tool for checking Soroban smart contract admin wallet types on the Stellar network.

## Overview

Admin Checker is a utility that helps identify whether a Soroban smart contract's admin wallet is a hot wallet or a multisig/cold wallet. This tool is useful for:

- Smart contract security audits
- Verifying contract ownership patterns
- Identifying potential security risks in contract administration

The tool works by:
1. Finding the admin address for a given contract
2. Determining if the admin is an EOA (Externally Owned Account) or another contract
3. For EOA admins, analyzing transaction patterns to determine if it's likely a hot wallet

## Installation

### Prerequisites

- Rust and Cargo

### Building from Source

```bash
git clone https://github.com/otter-sec/admin-checker.git
cd admin-checker
cargo build --release
```

## Usage

```bash
admin-checker [OPTIONS]
```

### Options

```
-c, --contract-id <CONTRACT_ID>    Target Contract ID
-a, --admin <ADMIN>                Admin key to search for. When used, all the other options are ignored
-r, --rpc-url <RPC_URL>            RPC URL to use. If you want to use a known or imported network, use the 'network' option instead
-n, --network <NETWORK>            Network to use. Available options are 'mainnet', 'testnet', 'futurenet', 'local' or 'standalone'. 
                                   Can also be used with custom networks, as long as these are imported in the local stellar-cli config
-k, --key <KEY>                    Admin's storage slot key to search for. Defaults to 'admin'
    --horizon <HORIZON>            Horizon URL to use. If not provided, it will be inferred from the network
```

### Examples

Check a contract on the testnet:
```bash
admin-checker --contract-id CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC --network testnet
```

Check a specific admin address:
```bash
admin-checker --admin GBXGQJWVLWOYHFLVTKWV5FGHA3LNYY2JQKM7OAJAUEQFU6LPCSEFVXON
```

Use a custom RPC URL:
```bash
admin-checker --contract-id CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC --rpc-url https://my-custom-soroban-rpc.example.com
```

## How It Works

Admin Checker performs the following steps:

1. **Contract Analysis**: Examines the contract's storage to find the admin address
   - Checks both instance storage and persistent storage
   - Supports various key formats (enum variants, symbols, strings)

2. **Admin Type Detection**: Determines if the admin is an EOA or another contract
   - For EOAs, proceeds to hot wallet analysis
   - For contracts, reports the contract status

3. **Hot Wallet Detection**: For EOA admins, analyzes transaction patterns
   - Checks if the account is not a multisig
   - Analyzes transaction frequency to determine if it's a hot wallet

## Networks

The tool supports the following networks out of the box:
- Mainnet
- Testnet
- Futurenet
- Local/Standalone

It can also use custom networks configured in your local Stellar CLI configuration.

## Configuration

`admin-checker` will check the following locations for network configurations, as per the default settings of the `stellar-cli`:
- `$XDG_CONFIG_HOME/stellar/network/`
- `$XDG_CONFIG_HOME/soroban/network/`
- `$HOME/.config/stellar/network/`
- `$HOME/.config/soroban/network/`

