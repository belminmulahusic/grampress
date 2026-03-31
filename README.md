# Grampress (gpress) ![Version](https://img.shields.io/badge/version-0.1.0-blue)


Grampress (gpress) is a simple command-line tool for grammar-based compression, decompression, and efficient search in compressed text data.

## Download Binary

You can download the latest release binary here:  

[![Download gpress](https://img.shields.io/badge/Download-gpress-blue)](https://git.thm.de/compress-group/grampress/-/jobs/artifacts/dev/raw/target/release/gpress?job=build_binary)



## Features

- Compress text using grammar-based compression
- Decompress `.gps` files
- Search for patterns in compressed files

## Usage

```bash
# Usage information
./gpress --help
```
```bash
# Compress / Decompress
./gpress text.txt
./gpress --decompress text.txt.gps
```

```bash
# Search for a pattern in a .gps file
./gpress text.txt
./gpress --search "pattern" text.txt.gps
```


