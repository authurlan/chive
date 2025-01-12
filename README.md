<div align="center">

# Cryptographic Hybrid Infrastructure for Visionary Ecosystems (CHIVE)

<img height="280px" alt="CHIVE Logo" src="https://github.com/authurlan/chive/raw/main/docs/images/chive_logo.png"/>

> **CHIVE** is a cutting-edge blockchain framework designed to build **Cryptographic Hybrid Infrastructure for Visionary Ecosystems**.  
> This project provides a robust foundation for creating a [parachain](https://wiki.polkadot.network/docs/learn-parachains) based on the Polkadot SDK, enabling seamless integration with the Polkadot ecosystem.

</div>

---

## Overview

* ⏫ **CHIVE** offers a powerful framework for building **Cryptographic Hybrid Infrastructure** tailored to visionary ecosystems.  
* ☁️ Built on the [Cumulus](https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/polkadot_sdk/cumulus/index.html) framework, it ensures interoperability and scalability.  
* 🔧 The runtime is configured with a custom pallet as a starting point, alongside essential pallets like the [Balances pallet](https://paritytech.github.io/polkadot-sdk/master/pallet_balances/index.html).  
* 👉 Learn more about parachains [here](https://wiki.polkadot.network/docs/learn-parachains).

---

## Project Structure

The **CHIVE** framework consists of:

* 💿 **[Node](./node/README.md)** - The binary application that runs the blockchain.  
* 🧮 **[Runtime](./runtime/README.md)** - The core logic of the parachain, defining its behavior.  
* 🎨 **[Pallets](./pallets/README.md)** - Modular components that construct the runtime.

---

## Prerequisites

* 🦀 **CHIVE** is built using the Rust programming language.  
* 👉 Follow the [Rust installation instructions](https://www.rust-lang.org/tools/install) for your system.  
* 🛠️ Additional dependencies may be required depending on your OS and Rust version. Check the Rust compiler output for details.

### Install Build Tools
To ensure the project builds successfully, you need to install the **build-essential** package, which includes the C compiler (`cc`) and other necessary tools:

```bash
sudo apt update
sudo apt install build-essential
```

Verify the installation by checking the GCC version:

```bash
cc --version
```

### Install Protocol Buffers Compiler
The project requires the **Protocol Buffers compiler** (`protoc`) to generate code from `.proto` files. Install it using the following command:

```bash
sudo apt install protobuf-compiler
```

Verify the installation by checking the `protoc` version:

```bash
protoc --version
```

### Install LLVM and Clang
The project requires **libclang** for certain dependencies (e.g., `librocksdb-sys`). Install LLVM and Clang using the following command:

```bash
sudo apt install llvm clang
```

Verify the installation by checking the Clang version:

```bash
clang --version
```

### Install Rust Toolchain and WASM Target
The project requires the **WASM compilation target** (`wasm32-unknown-unknown`) and the **stable Rust toolchain**. Install them using the following commands:

1. Install the WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown --toolchain stable-x86_64-unknown-linux-gnu
   ```

2. Install the stable Rust toolchain:
   ```bash
   rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu
   ```

3. Verify the installation:
   ```bash
   rustup target list --installed
   ```
---

### Build

🔨 To build the node without launching it, run:

```sh
cargo build --release
```

🐳 Alternatively, build the Docker image:

```sh
docker build . -t chive-parachain
```

---

### Local Development Chain

🧟 **CHIVE** uses [Zombienet](https://github.com/paritytech/zombienet) to orchestrate relaychain and parachain nodes.  
You can download a [released binary](https://github.com/paritytech/zombienet/releases/latest) or use the [npm version](https://www.npmjs.com/package/@zombienet/cli).

This project produces a parachain node. You will also need a relaychain node.  
Download the `polkadot`, `polkadot-prepare-worker`, and `polkadot-execute-worker` binaries from [Polkadot SDK releases](https://github.com/paritytech/polkadot-sdk/releases/latest).

Ensure the following binaries are in your `PATH`:

```sh
export PATH="./target/release/:$PATH"
```

👥 Start a local development chain with a single relaychain node and a parachain collator:

```sh
zombienet --provider native spawn ./zombienet.toml

# Alternatively, using the npm version:
npx --yes @zombienet/cli --provider native spawn ./zombienet.toml
```

Development chains:

* 🧹 Do not persist state.  
* 💰 Are preconfigured with a genesis state that includes prefunded development accounts.  
* 🧑‍⚖️ Development accounts are used as validators, collators, and `sudo` accounts.

---

### Connect with Polkadot-JS Apps

* 🌐 Interact with your local node using the hosted version of [Polkadot/Substrate Portal](https://polkadot.js.org/apps/):  
  - [Relay Chain](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)  
  - [Parachain](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9988)  

* 🪐 A hosted version is also available on [IPFS](https://dotapps.io/).  

* 🧑‍🔧 Host your own instance by following the instructions in the [`polkadot-js/apps`](https://github.com/polkadot-js/apps) repository.

---

## Contributing

* 🔄 This project is regularly updated with releases from the [Polkadot SDK monorepo](https://github.com/paritytech/polkadot-sdk).  
* ➡️ Submit pull requests to the [source repository](https://github.com/paritytech/polkadot-sdk/tree/master/templates/parachain).  
* 😇 Refer to the [contribution guidelines](https://github.com/paritytech/polkadot-sdk/blob/master/docs/contributor/CONTRIBUTING.md) and [Code of Conduct](https://github.com/paritytech/polkadot-sdk/blob/master/docs/contributor/CODE_OF_CONDUCT.md).

---

## Getting Help

* 🧑‍🏫 Learn more about Polkadot at [Polkadot.network](https://polkadot.network/).  
* 🧑‍🔧 Explore technical documentation in the [Polkadot SDK docs](https://github.com/paritytech/polkadot-sdk#-documentation).  
* 👥 Seek help via [GitHub issues](https://github.com/paritytech/polkadot-sdk/issues) or [Substrate StackExchange](https://substrate.stackexchange.com/).

---

## License

**CHIVE** is open-source and licensed under the [Apache 2.0 License](./LICENSE).
