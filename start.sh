#!/bin/sh
set -e
IPFS_DIR=/root/.ipfs

if [ ! -f $IPFS_DIR/config ]; then
  echo "[Satellite] Initialisation d'IPFS..."
  ipfs init
fi

echo "[Satellite] Configuration des adresses API, Swarm et Gateway..."
ipfs config Addresses.API /ip4/0.0.0.0/tcp/5001
ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080
ipfs config Addresses.Swarm '["/ip4/0.0.0.0/tcp/4001", "/ip4/0.0.0.0/udp/4001/quic"]' --json

echo "[Satellite] Lancement du daemon IPFS..."
ipfs daemon &

sleep 5

KC__SERVER__PEER_ID=$(ipfs id -f='<id>')
echo "[Satellite] PEER ID: $KC__SERVER__PEER_ID"

echo "[Satellite] Lancement du service KeystonCloud Satellite..."
cargo watch --env KC__SERVER__PEER_ID=$KC__SERVER__PEER_ID -p gateway -x run
