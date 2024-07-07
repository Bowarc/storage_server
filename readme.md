# Simple server that stores data

My goal was to make something like [transfer.sh](https://transfer.sh/) (which is probably down atm) or [wetransfer](https://wetransfer.com/)


// Compressed Delivery Network :D


## Status

- Backend
    It works well,
    I think cleanup is needed in the cache management code
    Routes are ugly but eh, it works
    I think logs are bad, and it needs better failure management

- Front-end
    It's really basic
    Some of the css is still done inline
    A good design is still needed
    But it works

## Roadmap
- [x] The actual server
    - [x] Web server that we can upload files to
    - [x] Web server that we can download files from
    - [x] Json api
- [x] Compression
- [x] Wasm front end
    - [x] Simple load and upload
    - [x] Simple download
- [ ] Integration with curl [#6](https://github.com/Bowarc/storage_server/issues/6)
- [ ] Simple download link [#7](https://github.com/Bowarc/storage_server/issues/7)

## Notes
Idk if any security is needed (ouside something against ddos or spam but i wont do that here)
About file size, we rly should set a limit, even like a rly high one, but a limit is needed

Store different infos in the json ?
Maybe use ron instead of json


## How to use
The scripts are in the ./scripts dir  
`sh scripts/build.sh`
