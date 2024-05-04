cargo zigbuild -r --target x86_64-unknown-linux-gnu.2.31
mkdir steamdeck_build
cp ./assets ./steamdeck_build/assets -R
cp ./target/x86_64-unknown-linux-gnu/release/svincraft ./steamdeck_build/svincraft
