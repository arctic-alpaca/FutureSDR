.PHONY: wasm wasm-debug wasm-profiling

wasm:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target no-modules --out-name fft_worker --out-dir ./dist --release --features include_main

wasm-debug:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target no-modules --out-name fft_worker --out-dir ./dist --debug --features include_main

wasm-profiling:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target no-modules --out-name fft_worker --out-dir ./dist --profiling --features include_main
