#! /bin/bash
cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "minesweeper" \
    ./target/wasm32-unknown-unknown/release/minesweeper.wasm

mv ./out/minesweeper_bg.wasm ./out/minesweeper_bg_non_opt.wasm

wasm-opt -Oz -o ./out/minesweeper_bg.wasm ./out/minesweeper_bg_non_opt.wasm
