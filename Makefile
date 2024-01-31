all:
	cargo build

wasm:
	cargo build --release --target wasm32-unknown-unknown && \
	wasm-bindgen target/wasm32-unknown-unknown/release/wasmcrash.wasm --out-dir ./www --target web
