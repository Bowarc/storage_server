# Simple server that stores data

My goal was to make something like [transfer.sh](https://transfer.sh/) (which is probably down atm) or [wetransfer](https://wetransfer.com/)


// Compressed Delivery Network :D


## Status

- Backend
    It works well.
    I think cleanup is needed in the cache management code.  
    Routes are ugly but eh, it works
    I think logs are bad, and it needs better failure management

- Front-end
    It's really basic
    Some of the CSS is still done inline
    A good design is still needed
    But it works

## Roadmap
- [x] The actual server
    - [x] Web server that we can upload files to
    - [x] Web server that we can download files from
    - [x] JSON API
- [x] Compression
- [x] WASM front end
    - [x] Simple load and upload
    - [x] Simple download
- [x] Integration with curl [#6](https://github.com/Bowarc/storage_server/issues/6)
- [x] Simple download link [#7](https://github.com/Bowarc/storage_server/issues/7)

## Notes
Idk if any security is needed (ouside something against DDoS or spam but i wont do that here)

About file size, we really should set a limit, even like a rly high one, but a limit is needed.  
(See the `file` default.limit in [Rocket.toml](./Rocket.toml))

## How to use
First, download the projects with
```console
git clone https://github.com/bowarc/storage_server
cd ./storage_server
```

### Build
In each build script (`./scripts`, you'll find `mode=debug # debug, release` at the top,  
replace `debug` with `release` to build a more optimized version of the program (build time will be slower)

Start by running `sh scripts/init.sh`  
This will create some important folders in the project directory, which the server relies on.

#### Build everything
`sh scripts/build.sh`

#### Build back
`sh scripts/build_back.sh`

#### Build front
`sh scripts/build_front.sh`

### Run
To run the server, use `sh scripts/run.sh`  
⚠️ Make sure the front it built, else the server wont be able to serve any web user

### CURL

#### Upload

```console
curl --upload_file ./file.ext http://<YOUR_ADDRESS:YOUR_PORT>/
```
This yields back an uuid that is used by the server to identify that file

#### Download

```console
curl http://<YOUR_ADDRESS:YOUR_PORT>/<UUID>/file.ext -O
```

> **_NOTE:_** On browser you only need the UUID as it auto redirects to the right file name (```http://<YOUR_ADDRESS:YOUR_PORT>/<UUID>``` -> ```http://<YOUR_ADDRESS:YOUR_PORT>/<UUID>/file.ext```).  
    Take a look at [#7](https://github.com/Bowarc/storage_server/issues/7) for more informations.

