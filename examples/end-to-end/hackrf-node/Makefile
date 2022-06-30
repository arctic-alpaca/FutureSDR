.PHONY: wasm serve

wasm:
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name hackrf_node --out-dir ./dist --release
	./gulp
	cd dist && ../serve.py

serve:
	cd dist && ../serve.py
