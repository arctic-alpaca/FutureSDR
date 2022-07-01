.PHONY: wasm serve

wasm:
	# building in a temporary directory and moving to the dist directory is required since wasm-opt fails when trying to optimize "hackrf_open.wasm"
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name hackrf_node --out-dir ./dist-tmp --release
	mv ./dist-tmp ./dist
	./gulp
	cd dist && ../serve.py

serve:
	cd dist && ../serve.py
