.PHONY: build wasm-profiling

build:
	RUSTFLAGS='-C target-cpu=generic' wasm-pack build --target web --out-name futuresdr --out-dir ./dist --release
	./gulp

wasm-profiling:
	RUSTFLAGS='-C target-cpu=generic' wasm-pack build --target web --out-name futuresdr --out-dir ./dist --profiling
	./gulp
