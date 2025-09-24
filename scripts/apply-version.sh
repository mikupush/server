#!/bin/bash

VERSION=$(cat VERSION)

sed -i.bak "3s/version = \".*\"/version = \"$VERSION\"/g" Cargo.toml

rm Cargo.toml.bak
