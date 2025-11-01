#!/bin/sh
ipfs daemon &
cargo watch -p gateway -x run
