.PHONY: cpu vulkan wasm wasm-profiling serve

wasm:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name recorded --out-dir ./dist --dev -- --features=wgpu
	cd ../../../frontend && make && ./gulp
	./gulp
	cd dist && ../serve.py

wasm-profiling:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name recorded --out-dir ./dist --profiling -- --features=wgpu
	cd ../../../frontend && make wasm-profiling && ./gulp
	./gulp
	cd dist && ../serve.py


serve:
	cd dist && ../serve.py

cpu:
	cargo run --bin=cpu --features=cpu

vulkan:
	cargo run --bin=vulkan --features=cpu,vulkan

