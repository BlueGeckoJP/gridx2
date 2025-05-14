# gridx2

A modern, fast image viewer built with GTK4 and Rust. gridx2 provides an efficient way to browse and view images in directories with an accordion-style interface.

## Features

- 🖼️ **Directory-based Image Viewing**: Browse images organized by directories in an expandable accordion interface
- 🚀 **Fast Image Loading**: Parallel image processing and LRU caching for optimal performance
- 📂 **Recursive Directory Support**: Configurable depth for recursive directory scanning
- 🎯 **Thumbnail Support**: Customizable thumbnail sizes for better preview
- 🔍 **Natural Sorting**: Intelligent file sorting for better organization
- 💫 **Modern UI**: Built with GTK4 for a sleek, native look and feel

## Installation

### Prerequisites

- Rust (Tested only on nightly)
- GTK4 development libraries

### Building from Source

1. Clone the repository

```bash
git clone https://github.com/BlueGeckoJP/gridx2.git
cd gridx2
```

2. Build and run

```bash
cargo build --release
./target/release/gridx2
```

## Usage

1. Launch the application
2. Use the File menu to:
   - Open a folder containing images
   - Access settings
3. Click on directories in the accordion view to load and view images
4. Use the settings window to configure:
   - Thumbnail size
   - Maximum directory depth
   - Image opening command

## Supported Image Formats

Please refer to the decoding section below for the supported image formats.

[Supported Image Formats](https://github.com/image-rs/image?tab=readme-ov-file#supported-image-formats)

## Performance Features

- Parallel image processing using Rayon
- LRU caching for faster image loading
- Lazy loading of images when expanding directories
- Progress bar for loading feedback

## License

This project is open source and available under the [MIT License](https://github.com/BlueGeckoJP/gridx2/blob/master/LICENSE).
