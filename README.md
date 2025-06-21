# Docker Image Search

Table of Contents

- [Overview](#overview)
- [Usage](#usage)
  - [Installation](#installation)
  - [Environment](#environment)
  - [Image Search Examples](#image-search-examples)
    - [Redis](#redis)
    - [Foundry](#foundry)
    - [Ethereum's Golang Client](#ethereums-golang-client)
  - [Caveats](#caveats)
    - [Platform](#platform)
    - [Webpage](#webpage)
    - [Private Repos](#private-repos)
    - [Local Repos](#local-repos)
    - [References / Tags](#references--tags)
    - [Docker Hub](#docker-hub)
  - [Github](#github)
- [Development](#development)
  - [Architecture](#architecture)
  - [Tests](#tests)

## Overview

If you've ever looked at a Dockerfile and wanted to know how one of its images is built but didn't know where to find it then you can query this tool with the image and it will **attempt** to find and display the Dockerfile in your default browser.

> Note: Work in progress

## Usage

### Installation

This is a [Rust](https://www.rust-lang.org/) based tool so you must have **cargo** installed.

```shell
$ cargo install --path .
```

Verify installation

```shell
$ dis --help

Usage: dis <IMAGE> <USER> <TOKEN>

Arguments:
  <IMAGE>  Docker image name with optional tag (e.g., project:reference)
  <USER>   GitHub username [env: GITHUB_USER]
  <TOKEN>  GitHub token with read access to packages [env: GITHUB_TOKEN]

Options:
  -h, --help  Print help
```

### Environment

To search for references on GitHub we require a PAT with read access to packages. You may either provide it in the cli as an arg, set it as an environment variable or copy [.env.example](./.env.example) to `.env` and store your username/token in there (not recommended).

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

### Caveats

#### Platform

Linux only. May implement a Dockerfile if I feel like it but probably not so fork or PR the file.

#### Webpage

Default to displaying the closest page it can find if it cannot locate the Dockerfile but knows the image is somewhere in the project.

#### Private Repos

No authentication so private projects may not work.

#### Local Repos

Haven't tested. Probably doesn't work.

#### References / Tags

Not implemented everywhere so it may ignore it in some cases.

#### Docker Hub

The API does not expose a way to associate a reference with a project to allow us to find the source and open the Dockerfile directly. Webscraping is unreliable and a bad solution.

Once Docker Hub is opened you must read the overview and click the reference to redirect you to the Dockerfile, if the reference is listed and hyperlinked. 

### Github

If a reference is provided it will try to use it. If it fails to find the file for that reference it will attempt to use the default branch of the repo to find a file instead.

To actually use a reference GitHub requires a token which has read access to packages otherwise api queries are rejected.

> Not currently implemented for images with ghcr.io in them

## Development

### Architecture

TODO: update this terrible list

1. Given an image use docker to check if you have that image and inspect its contents to see if it has embedded labels that can be used to go straight to the website.
2. Break the image into components and determine potential origin i.e. Docker Hub, GitHub, some other registry
3. Using the components query to see if the file exists
   1. If exists, open directly
   2. If it doesn't exist but determined the image is somewhere in that project then open project instead

### Tests

Run the unit tests.

```shell
$ cargo test
```
