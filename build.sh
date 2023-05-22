wasm-pack build --target web --out-name castle_sim --out-dir ../CastleSimWeb/src/assets/wasm --release
cp -r ./assets ../CastleSimWeb/src/
rm ../CastleSimWeb/src/assets/wasm/.gitignore
