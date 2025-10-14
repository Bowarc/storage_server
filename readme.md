# Simple storage server with a wasm front end

// Compressed data node :D

## Goal

The goal of this project is to make a db-like local storage system for files.  

<u>**It's therefore not meant to be user-facing.**</u>  

## Status

- Backend  
    It works well.  
    Streaming compression and decompression makes it really fast and memory-efficient

    Could really use a dashboard system

- Front-end  
    Uses streaming for upload so it's fast  
    A good front-end design is still needed but not required ()
    But it works  

## Roadmap
- [x] The actual server
    - [x] Web server that we can upload files to
    - [x] Web server that we can download files from
    - [x] Streaming upload, download and compression
    - [x] Integration with curl [#6](https://github.com/Bowarc/storage_server/issues/6)
    - [x] Simple download link [#7](https://github.com/Bowarc/storage_server/issues/7)
    - [x] A way to not store duplicates using hash-based duplicate detection
            The implementation isn't the prettyest not the safest but it works
            (I'll rework it soon™)
    - [x] A way to delete a stored file (see [#3](https://github.com/Bowarc/storage_server/issues/3))
- [x] WASM front end
    - [x] Homepage
    - [x] Upload 

## Notes

About input file size, I've set 1Gib, but it's easy to modify  
(See `default.limit.file` in [Rocket.toml](./Rocket.toml))

## Installation

### Docker install

#### Download the git repo

```console
git clone https://github.com/bowarc/storage_server
cd ./storage_server
```
#### Build it

```console
sh scripts/docker_build.sh
```

#### Deploy it
Use host network and link a docker volume named 'storage_server' that points to the server's storage cache 
```console
docker run -d --network host -v storage_server:/app/cache storage_server:latest 
```

### Manual install

#### First, download the projects with

```console
git clone https://github.com/bowarc/storage_server
cd ./storage_server
```

In each build script `./scripts/build*`, you can specify the command line argument `r` or `release` to build the project in release mode  
This will enable some optimisations but make the compilation a bit slower.

#### Init
Start by running `sh scripts/init.sh`  
This will create some important folders in the project directory, which the server relies on.


#### Build back
`sh scripts/build_back.sh`

#### Build front
`sh scripts/build_front.sh`

#### Or Build everything with one command
`sh scripts/build.sh`

### Run
To run the server, use `sh scripts/run.sh`  
⚠️ Make sure the front it built, else the server wont be able to serve any web user

## Usage

Check the [examples](./examples) directory for one using python (make sure the server is running and you generated the sample file before running the example)

Any programming language able to make local web request could use it, here is an example using curl

#### Upload

```console
curl --upload_file ./file.ext http://<YOUR_ADDRESS:YOUR_PORT>/
```
This yields back an uuid that is used by the server to identify that file

#### Download

```console
curl http://<YOUR_ADDRESS:YOUR_PORT>/<UUID>/file.ext -O
```

#### Delete a file
```console
curl http://<YOUR_ADDRESS:YOUR_PORT>/<UUID> -X DELETE
```

> **_NOTE:_** On browser you only need the UUID as it auto redirects to the right file name  
(```http://<YOUR_ADDRESS:YOUR_PORT>/<UUID>``` -> ```http://<YOUR_ADDRESS:YOUR_PORT>/<UUID>/file.ext```).  
    Take a look at [#7](https://github.com/Bowarc/storage_server/issues/7) for more informations.
