#!/usr/bin/env bash

# release or debug
build_type="$1"
arch="$(dpkg --print-architecture)"
version="$(cat VERSION)"
target_directory="target/$build_type"
bundle_directory="$target_directory/bundle"
tar_package_directory="$bundle_directory/tar"

if [[ -d "$tar_package_directory" ]]; then
  rm -r "$tar_package_directory"
  rm -f "$bundle_directory/*.tar.gz"
fi

mkdir -p "$tar_package_directory"

cp "$target_directory/mikupush-server" "$tar_package_directory/mikupush-server"
cp package/mikupush-server.service "$tar_package_directory/mikupush-server.service"
cp -r static "$tar_package_directory/"
cp -r templates "$tar_package_directory/"
cp config.default.yaml "$tar_package_directory/config.yaml"

chmod +x "$tar_package_directory/mikupush-server"

tar \
  -C "$tar_package_directory" \
  -czvf "$bundle_directory/mikupush-server-linux-$version-$arch.tar.gz" .
