# Docker Image Search

Table of Contents

- [Overview](#overview)
- [Usage](#usage)
  - [Installation](#installation)
  - [Image Search Examples](#image-search-examples)
    - [Redis](#redis)
    - [Foundry](#foundry)
    - [Ethereum's Golang Client](#ethereums-golang-client)
- [Development](#development)

## Overview

If you've ever looked at a Dockerfile and wanted to know how one of its images is built but didn't know where to find it then you can query this tool with the image and it will **attempt** to find and display the Dockerfile in your default browser.

> NOTE: WIP, doesn't cover all formats, private repos you do not have access to, and may default to displaying the closest page it can find if it cannot locate the actual file but knows the image is somewhere in the project. It probably doesn't work across all platforms.

## Usage

### Installation

This is a [Rust](https://www.rust-lang.org/) based tool so you must have **cargo** installed.

```shell
$ cargo install --path .
```

Verify installation

```shell
$ dis --help

Usage: dis <IMAGE>

Arguments:
  <IMAGE>  Docker image name with optional tag (e.g., project:reference)

Options:
  -h, --help  Print help
```

### Image Search Examples

Here are some randomly selected images as examples of usage.

#### Redis

Open Docker Hub to find Redis.

```shell
$ dis redis
```

#### Foundry

Open GitHub to find Foundry.

```shell
$ dis foundry-rs/foundry
```

#### Ethereum's Golang Client

Open Docker Hub to find the client.

```shell
$ dis ethereum/client-go
```

## Development

Run the unit tests.

```shell
$ cargo test
```
