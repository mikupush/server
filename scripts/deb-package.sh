#!/usr/bin/env bash

# release or debug
build_type="$1"
arch="$(dpkg --print-architecture)"
version="$(cat VERSION)"
target_directory="target/$build_type"
bundle_directory="$target_directory/bundle"
deb_package_directory="$bundle_directory/deb"

if [[ -d "$bundle_directory" ]]; then
  rm -r "$bundle_directory"
fi

mkdir -p "$deb_package_directory/DEBIAN"
mkdir -p "$deb_package_directory/usr/bin"
mkdir -p "$deb_package_directory/usr/share/io.mikupush.server"
mkdir -p "$deb_package_directory/etc/io.mikupush.server"
mkdir -p "$deb_package_directory/etc/systemd/system"

control=$(cat <<EOF
Package: mikupush-server
Architecture: $arch
Version: $version
Section: web
Priority: optional
Maintainer: Miku Push! Team <mikupush.io@gmail.com>
Description: The Miku Push! server
Homepage: https://mikupush.io
EOF
)

echo "$control" > "$deb_package_directory/DEBIAN/control"
cp package/postinst.sh "$deb_package_directory/DEBIAN/postinst"
cp package/prerm.sh "$deb_package_directory/DEBIAN/prerm"
cp package/postrm.sh "$deb_package_directory/DEBIAN/postrm"
cp "$target_directory/mikupush-server" "$deb_package_directory/usr/bin/mikupush-server"
cp package/mikupush-server.service "$deb_package_directory/etc/systemd/system/mikupush-server.service"
cp -r static "$deb_package_directory/usr/share/io.mikupush.server/"
cp -r templates "$deb_package_directory/usr/share/io.mikupush.server/"
cp -r config.default.yaml "$deb_package_directory/etc/io.mikupush.server/config.yaml"

chmod +x "$deb_package_directory/usr/bin/mikupush-server"
chmod +x "$deb_package_directory/DEBIAN/postinst"
chmod +x "$deb_package_directory/DEBIAN/prerm"
chmod +x "$deb_package_directory/DEBIAN/postrm"

dpkg-deb --build "$deb_package_directory" "$target_directory/bundle/mikupush-server-$arch.deb"
