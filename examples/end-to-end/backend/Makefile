.PHONY: backend-debug backend-release end-to-end

backend-debug:
	cargo run --debug

backend-release:
	cargo run --release

end-to-end:
	mkdir -p serve/frontend
	mkdir -p serve/hackrf_node
	mkdir -p serve/recorded_node
	mkdir -p serve/css
	# building in a temporary directory and moving to the dist directory is required since wasm-opt fails when trying to optimize "hackrf_open.wasm"
	cd ../hackrf-node && \
		RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name hackrf_node --out-dir ./dist-tmp --release && \
		mv ./dist-tmp/* ./dist && \
		./gulp && \
		cp -r ./dist/* ../backend/serve/hackrf_node
	cd ../recorded-node && \
		RUSTFLAGS='-C target-cpu=generic --cfg=web_sys_unstable_apis' wasm-pack build --target web --out-name recorded_node --out-dir ./dist --release && \
		./gulp && \
		cp -r ./dist/* ../backend/serve/recorded_node
	cd ../frontend && \
		cp -r ./dist/* ../backend/serve/frontend
	cp serve/frontend/favicon.png serve/favicon.png
	cp serve/frontend/css/* serve/css
	./scripts/init_db.sh
	cargo run --release

clean-end-to-end:
	rm -rf serve
	cd ../hackrf-node && cargo clean
	cd ../recorded-node && cargo clean
	cargo clean
