.PHONY: wasm wasm-debug wasm-profiling serve

wasm:
	# building in a temporary directory and moving to the dist directory is required since wasm-opt fails when trying to optimize "hackrf_open.wasm"
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name hackrf_node_webworker --out-dir ./dist-tmp --release
	mv ./dist-tmp/* ./dist
	cd ../fft_worker && \
		make wasm && \
		cp -r ./dist/* ../hackrf-node-webworker/dist
	./gulp
	cd dist && ../serve.py

wasm-debug:
	# building in a temporary directory and moving to the dist directory is required since wasm-opt fails when trying to optimize "hackrf_open.wasm"
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name hackrf_node_webworker --out-dir ./dist-tmp --debug
	mv ./dist-tmp/* ./dist
	cd ../fft_worker && \
		make wasm-debug && \
		cp -r ./dist/* ../hackrf-node-webworker/dist
	./gulp
	cd dist && ../serve.py

wasm-profiling:
	# building in a temporary directory and moving to the dist directory is required since wasm-opt fails when trying to optimize "hackrf_open.wasm"
	RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name hackrf_node_webworker --out-dir ./dist-tmp --profiling
	mv ./dist-tmp/* ./dist
	cd ../fft_worker && \
		make wasm-profiling && \
		cp -r ./dist/* ../hackrf-node-webworker/dist
	./gulp
	cd dist && ../serve.py

serve:
	cd dist && ../serve.py
