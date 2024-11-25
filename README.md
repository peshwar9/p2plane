## P2PLane - A Peer-to-Peer Networking Library
##### A flexible and extensible P2P networking foundation for distributed applications

### Design Document
[Design Document](./docs/Design.md)

### Quick Start

```shell
# Clone the repository
git clone https://github.com/yourusername/p2plane.git
cd p2plane

# Build the project
cargo build
```

### Running Tests

```shell
# Run unit tests
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run all tests with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_name
```

### Running Examples

```shell
# Run the Narwhal consensus example
cd examples/narwhal
cargo run -- --port 8000  # Bootstrap node
cargo run -- --port 8001 --bootstrap "/ip4/127.0.0.1/tcp/8000"  # Peer node
```

### Features
- Peer-to-peer networking using libp2p
- Custom message type support via traits
- Distributed Hash Table (DHT) for peer discovery
- Asynchronous message processing
- Flexible network behavior configuration
- Built-in peer management
- Type-safe message handling

### Example Usage
```rust
use p2plane::{
    network::{Node, NodeConfig},
    traits::Message,
};

// Define your message type
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyMessage(String);

impl Message for MyMessage {
    fn protocol_id(&self) -> &'static str {
        "/my-app/1.0.0"
    }
}

// Create and start a node
async fn run_node() -> Result<(), Box<dyn Error>> {
    let config = NodeConfig {
        listen_addr: "/ip4/127.0.0.1/tcp/8000".parse()?,
        bootstrap_addr: None,
    };

    let node = Node::<MyMessage>::new(config).await?;
    Ok(())
}
```

### Contributing
We welcome contributions to P2PLane! Here's how you can contribute:
1. First, discuss the change you wish to make via a GitHub issue
2. Fork the repository and create your feature branch
3. Implement your changes with appropriate tests
4. Submit a Pull Request with a comprehensive description of changes

Please ensure your PR:
- Includes relevant tests
- Updates documentation as needed
- Follows the existing code style
- Has a clear and descriptive commit message

### License

MIT License
Copyright (c) 2024 
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
