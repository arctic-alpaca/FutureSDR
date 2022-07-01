.PHONY: wasm wasm-profiling

wasm:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name node --out-dir ./dist --dev
	./gulp
	cd dist && ../serve.py

wasm-profiling:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name node --out-dir ./dist --profiling
	./gulp
	cd dist && ../serve.py
