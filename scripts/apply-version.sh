#!/bin/bash

VERSION=$(cat VERSION)

sed -i.bak "3s/version = \".*\"/version = \"$VERSION\"/g" Cargo.toml
sed -i.bak "s/SERVER_VERSION: \&str = \".*\"/SERVER_VERSION: \&str = \"$VERSION\"/g" src/main.rs

echo "Remember to update SERVER_VERSION_CODE in main.rs"

rm Cargo.toml.bak
rm src/main.rs.bak
