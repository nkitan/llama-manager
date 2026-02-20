# Llama Manager

A powerful, feature-rich desktop application for managing and configuring Llama language models with an intuitive graphical interface. Built with Rust using the Dioxus framework and WebView2 for a modern, responsive user experience.

## Overview

Llama Manager simplifies the process of running and configuring large language models locally. With a comprehensive UI, you can fine-tune every aspect of your Llama model server, from GPU memory allocation to advanced sampling parameters, all without touching command-line arguments.

### Key Features

- **Tabbed Interface**: Organized settings across 8 specialized tabs for different configuration aspects
- **Model Management**: Load and configure various Llama model parameters
- **Server Controls**: Configure network settings, ports, and server options
- **GPU/Memory Optimization**: Fine-tune GPU allocation, memory management, and KV cache settings
- **Performance Tuning**: Adjust thread counts, batch sizes, and processing speed parameters
- **Sampling Control**: Configure temperature, top-p, top-k, and other sampling strategies
- **Advanced Settings**: Access to advanced parameters like RoPE scaling, split modes, and attention options
- **API Security**: Manage API keys, authentication, and security-related configurations
- **Context Configuration**: Fine-tune context window sizes and related parameters
- **Real-time Server Management**: Start, stop, and monitor your Llama server directly from the app

## System Requirements

- **OS**: Windows (using WebView2), Linux, or macOS
- **Rust**: 1.70 or later
- **WebView2**: Pre-installed on modern Windows systems
- **Memory**: Minimum 8GB RAM (recommended 16GB+ for larger models)
- **GPU** (optional): NVIDIA GPU with CUDA support for accelerated inference

## Installation

### Prerequisites

Ensure you have Rust installed. If not, download it from [rustup.rs](https://rustup.rs/).

### Building from Source

1. Clone the repository:
```bash
git clone <repository-url>
cd llama-manager
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

## Project Structure

```
llama-manager/
├── Cargo.toml              # Project manifest with dependencies
├── README.md               # This file
└── src/
    ├── main.rs            # Application entry point, UI components, and event handling
    └── config.rs          # Configuration structures, enums, and serialization logic
```

### Main Components

#### `main.rs`
The core application file containing:
- **Tab Enum**: Defines 8 configuration tabs (Model, Server, Context, GPU, Performance, Sampling, Advanced, API)
- **UI Components**: Dioxus components for rendering the interface
- **Event Handlers**: Logic for user interactions and server communication
- **CSS Styling**: Comprehensive styling for a modern dark-themed UI
- **Server Management**: Integration with Llama server processes

#### `config.rs`
Configuration management with:
- **Enum Definitions**: 
  - `SplitMode`: Layer, Row, or None
  - `CacheType`: F16, Q8_0, or Q4_0
  - `RopeScaling`: None, Linear, or Yarn
  - `LogFormat`: Text or JSON
  - `PoolingType`: Various pooling strategies
  - And more...
- **Serialization Support**: Full serde integration for saving/loading configurations
- **Type Safety**: Strongly-typed configuration with conversion methods

## Configuration Tabs

### 📦 Model Tab
- Model path and selection
- Model-specific parameters
- Quantization settings
- Model loading options

### 🌐 Server Tab
- Server host and port configuration
- Network settings
- Server startup options
- API endpoints configuration

### 📏 Context Tab
- Context window size
- Context-related parameters
- Sequence length settings
- Context management options

### 🎮 GPU & Memory Tab
- GPU memory allocation
- KV cache configuration
- Memory type selection
- VRAM management
- Batch and ubatch sizing

### ⚡ Performance Tab
- Thread allocation
- Processing optimization
- Speed vs quality trade-offs
- Parallel processing settings

### 🎲 Sampling Tab
- Temperature control
- Top-p (nucleus sampling)
- Top-k filtering
- Frequency and presence penalties
- Mirostat sampling configuration

### 🔧 Advanced Tab
- RoPE scaling methods
- Attention optimization
- Split modes for layer distribution
- Advanced memory and computation options
- Fine-grained model behavior tuning

### 🔒 API & Security Tab
- API key management
- Authentication configuration
- Security headers
- Access control settings
- CORS configuration

## Dependencies

The project uses the following key dependencies:

- **dioxus** (0.7.3): Reactive UI framework for Rust
- **dioxus-desktop** (0.7.3): Desktop integration for Dioxus
- **serde** (1.x): Data serialization/deserialization
- **serde_json** (1.x): JSON support
- **rfd** (0.17): File dialogs for model selection
- **tokio** (1.x): Async runtime for concurrent operations
- **reqwest** (0.12): HTTP client for API communication

## Usage

### Starting the Application

```bash
# Development
cargo run

# Release (optimized)
cargo run --release
```

### Basic Workflow

1. **Load a Model**: Navigate to the Model tab and select your Llama model file
2. **Configure Server**: Set the server host, port, and networking options
3. **Optimize Performance**: Adjust GPU memory, thread counts, and batch sizes based on your hardware
4. **Set Sampling Parameters**: Configure temperature and sampling methods for your use case
5. **Fine-tune Advanced Settings**: Adjust advanced parameters if needed
6. **Start Server**: Use the server controls to launch your Llama instance
7. **Monitor & Adjust**: Watch performance metrics and adjust parameters as needed

### Configuration Persistence

Configurations are automatically saved to allow quick resumption of previous setups. The application supports loading and saving configuration profiles.

## Development

### Project Structure for Developers

The codebase is organized for clarity and maintainability:

- **UI Logic** (`main.rs`): All visual components and user interactions
- **Data Management** (`config.rs`): Configuration structures and business logic
- **Separation of Concerns**: UI and configuration are cleanly separated

### Building for Development

```bash
# Debug build (faster compilation)
cargo build

# Run with debug output
RUST_LOG=debug cargo run
```

### Building for Release

```bash
# Optimized release build
cargo build --release

# Output location
./target/release/llama-manager.exe
```

## Features in Detail

### Real-time Server Management
- Start and stop Llama servers from the UI
- View server logs and status
- Configure pre-launch settings

### Comprehensive Parameter Control
- 100+ configurable parameters
- Organized into logical groups
- Safety checks for invalid configurations

### Model Flexibility
Support for various Llama model variants with different:
- Quantization levels (F16, Q8_0, Q4_0)
- Context window sizes
- Memory configurations
- Performance profiles

### Advanced Optimization
- RoPE scaling for extended context
- Layer splitting strategies
- Memory optimization modes
- Batch size tuning

## Troubleshooting

### Common Issues

**WebView2 Not Found**
- Llama Manager uses WebView2 for rendering
- Install it from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

**Server Won't Start**
- Check that the model file path is correct
- Verify sufficient GPU/system memory
- Check server port is not already in use

**High Memory Usage**
- Reduce context window size
- Lower batch sizes (batch/ubatch)
- Use quantized model formats (Q8_0, Q4_0)

**Poor Performance**
- Adjust thread count for your CPU
- Enable GPU acceleration if available
- Optimize batch sizes for your hardware

## Performance Tips

1. **Match batch size to your GPU memory**: Larger batches = faster but more VRAM
2. **Use quantization**: Q4_0 provides good balance of quality and speed
3. **Adjust context size**: Larger context = higher quality but slower and more memory
4. **Optimize thread allocation**: Match to your CPU core count
5. **Monitor temperatures**: Ensure hardware doesn't thermal throttle

## License

[Add your license information here]

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Future Enhancements

- Model comparison tools
- Performance benchmarking suite
- Configuration templates for popular models
- Batch inference support
- Fine-tuning capabilities
- Web UI dashboard
- Cross-platform installation packages
- Configuration import/export utilities
- A/B testing for sampling parameters

## Support

For issues, questions, or feature requests, please open an issue on the project repository or contact the development team.

---

**Version**: 0.1.0  
**Last Updated**: February 2026  
**Status**: Active Development
