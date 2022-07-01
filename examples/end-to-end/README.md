# FutureSDR End-to-End Example
This example showcases a node processing SDR-data via FutureSDR, then sending it to a backend. The backend serves
all css/html/js/wasm files needed and relays data coming from the node to one connected frontend. The frontend
visualizes incoming data. A PostgreSQL database is started in a container and the backend stores incoming node
data in it.

## Requirements
* rust/cargo
* FutureSDR Requirements
* podman
* psql
* sqlx CLI
* make
* The HackRF node requires a connected HackRF SDR.

## Usage
Run `npm install` once in the [hackrf-node](hackrf-node) and the [recorded-node](recorded-node) directory.
Use the `Makefile` in [backend/Makefile](backend/Makefile) to build and start the end-to-end example:
* To start: `make end-to-end`
* To clean: `make clean-end-to-end`

When the example is running, you can navigate to:
* hackrf_node: [127.0.0.1:3000/hackrf_node/](http://127.0.0.1:3000/hackrf_node/)
* recorded_node: [127.0.0.1:3000/recorded_node/](http://127.0.0.1:3000/recorded_node/)
* frontend: [127.0.0.1:3000/frontend/](http://127.0.0.1:3000/frontend/)

## Limitations
* Opening multiple node tabs is currently not supported, data will be show interleaved in the frontend.
* Opening multiple frontend tabs is corrently not supported, data will be split between frontends.
* The node tab has to be visible, otherwise the example can get throttled (hackrf_node) or completely paused (recorded_node).
* The example has only been tested on Chromium

## Database
The incoming data is saved in a PostgreSQL database but currently can't be replayed.
