# Solana Vanity Address Generator

A high-performance Rust-based tool to generate Solana vanity addresses with custom prefix and/or suffix patterns.

## Features

- 🚀 **Multi-threaded**: Utilizes all CPU cores for maximum performance
- 🎯 **Flexible Matching**: Support for prefix, suffix, or both
- 📊 **Real-time Progress**: Optional verbose mode to track generation speed
- ✅ **Base58 Validation**: Ensures only valid Base58 characters are used
- 🔒 **Secure**: Generates cryptographically secure keypairs

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Build from Source

```bash
git clone <repository-url>
cd vanity-address-solana
cargo build --release
```

The compiled binary will be available at `target/release/vanity-address-solana`

### Install Globally (Recommended)

To install the binary globally so you can run it from anywhere:

```bash
cargo install --path .
```

After installation, you can run the tool directly:

```bash
vanity-address-solana --prefix "ABC"
```

### Alternative: Add to PATH

You can also add the release directory to your PATH:

```bash
# macOS/Linux - Add to ~/.zshrc or ~/.bashrc
export PATH="$PATH:$(pwd)/target/release"
```

Then run:

```bash
vanity-address-solana --prefix "ABC"
```

## Usage

### Quick Start (After Building)

If you built with `cargo build --release`:
```bash
# Run from project directory
./target/release/vanity-address-solana --prefix "ABC"

# Or using cargo run (slower, but convenient during development)
cargo run --release -- --prefix "ABC"
```

If you installed globally with `cargo install --path .`:
```bash
vanity-address-solana --prefix "ABC"
```

### Basic Commands

Generate an address with a specific prefix:
```bash
cargo run --release -- --prefix "ABC"
```

Generate an address with a specific suffix:
```bash
cargo run --release -- --suffix "xyz"
```

Generate an address with both prefix and suffix:
```bash
cargo run --release -- --prefix "Sol" --suffix "123"
```

Generate multiple addresses:
```bash
cargo run --release -- --prefix "ABC" --count 5
```

Generate with case-sensitive matching:
```bash
cargo run --release -- --prefix "ABC" --case-sensitive
```

### Advanced Options

Specify number of threads:
```bash
cargo run --release -- --prefix "ABC" --threads 8
```

Enable verbose mode to see real-time attempts:
```bash
cargo run --release -- --prefix "ABC" --verbose
```

Combine multiple options:
```bash
cargo run --release -- --prefix "Sol" --suffix "end" --count 3 --case-sensitive --verbose --threads 16
```

### Command-line Arguments

- `-p, --prefix <PREFIX>`: Specify the prefix pattern to match
- `-s, --suffix <SUFFIX>`: Specify the suffix pattern to match
- `-n, --count <COUNT>`: Number of addresses to generate (default: 1)
- `-t, --threads <THREADS>`: Number of threads to use (default: number of CPU cores)
- `-c, --case-sensitive`: Enable case-sensitive matching (default: case-insensitive)
- `-v, --verbose`: Show attempts per second in real-time
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Examples

### Example 1: Simple Prefix Search

```bash
cargo run --release -- --prefix "Dog"
```

Output:
```
🔍 Searching for Solana vanity address with prefix 'Dog'
⚙️  Using 8 threads
⏳ This may take a while...

✅ Found vanity address!

📍 Public Key:  DogXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
🔑 Private Key: [1, 2, 3, ...]

📊 Statistics:
   Attempts: 458234
   Time: 12.34s
   Rate: 37132 attempts/sec

⚠️  IMPORTANT: Save your private key securely!
   You can import it using: solana-keygen recover
```

### Example 2: Suffix Search with Verbose Mode

```bash
cargo run --release -- --suffix "end" --verbose
```

### Example 3: Combined Prefix and Suffix

```bash
cargo run --release -- --prefix "Sol" --suffix "Dev"
```

### Example 4: Generate Multiple Addresses

```bash
cargo run --release -- --prefix "ABC" --count 10 --verbose
```

Output:
```
🔍 Searching for 10 Solana vanity addresses with prefix 'abc' (case-insensitive)
⚙️  Using 8 threads
⏳ This may take a while...

⚡ Attempts: 2543821 | Found: 10/10 | Rate: 42397 attempts/sec
✅ Found 10 vanity addresses!

──────────────────────────────────────────────
Address #1
📍 Public Key:  AbcXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
🔑 Private Key: [1, 2, 3, ...]

──────────────────────────────────────────────
Address #2
📍 Public Key:  aBcYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
🔑 Private Key: [4, 5, 6, ...]

...

📊 Statistics:
   Total Attempts: 2543821
   Addresses Found: 10
   Time: 60.00s
   Rate: 42397 attempts/sec
   Avg per address: 254382 attempts

⚠️  IMPORTANT: Save your private keys securely!
   You can import them using: solana-keygen recover
```

## Important Notes

### Matching Modes

- **Case-insensitive (default)**: Matches patterns regardless of case. For example, `--prefix "abc"` will match `Abc`, `ABC`, `aBc`, etc.
- **Case-sensitive**: Use `--case-sensitive` flag to match exact case. For example, `--prefix "ABC" --case-sensitive` will only match addresses starting with uppercase `ABC`.

**Note**: Case-insensitive mode significantly increases your chances of finding a match since it accepts any combination of upper/lowercase letters.

### Security

- **Keep your private key secure**: Never share your private key with anyone
- **Back up your keys**: Store them in a secure location
- The private key is displayed in array format for easy import

### Performance

- Longer patterns take exponentially more time to find
- Each additional character increases difficulty by ~58x (Base58 alphabet)
- Using more threads generally improves performance

### Base58 Alphabet

Valid characters: `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`

Note: The following characters are NOT valid in Base58:
- `0` (zero)
- `O` (uppercase o)
- `I` (uppercase i)
- `l` (lowercase L)

## Difficulty Estimation

| Pattern Length | Approximate Attempts | Estimated Time (50k/sec) |
|----------------|---------------------|--------------------------|
| 1 character    | ~29                 | < 1 second              |
| 2 characters   | ~1,682              | < 1 second              |
| 3 characters   | ~97,336             | ~2 seconds              |
| 4 characters   | ~5.6M               | ~2 minutes              |
| 5 characters   | ~328M               | ~1.8 hours              |
| 6 characters   | ~19B                | ~4.4 days               |

*Note: These are rough estimates. Actual time depends on CPU performance and luck.*

## Troubleshooting

### Error: "Invalid Base58 character"

Make sure your prefix/suffix only contains valid Base58 characters. Avoid `0`, `O`, `I`, and `l`.

### Low Performance

1. Make sure you're using the `--release` flag when building/running
2. Try adjusting the number of threads with `--threads`
3. Close other CPU-intensive applications

## License

MIT License

## Disclaimer

This tool generates cryptographically secure keypairs, but the security of your funds depends on how you store and manage your private keys. Always follow best practices for key management.
