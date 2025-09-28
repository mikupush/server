#!/bin/bash

USER="mikupush"

mkdir -p /var/lib/io.mikupush.server/data
mkdir -p /var/log/io.mikupush.server

if ! id -u "$USER" > /dev/null 2>&1; then
    useradd -s /usr/sbin/nologin "$USER"
fi

chmod +x /usr/bin/mikupush-server
chmod 775 /etc/io.mikupush.server
chmod -R 0664 /etc/io.mikupush.server/*
chmod 775 /var/lib/io.mikupush.server
chmod 775 /var/log/io.mikupush.server
chmod -R 0664 /var/log/io.mikupush.server/*
chmod 775 /usr/share/io.mikupush.server
chmod -R 0664 /usr/share/io.mikupush.server/*
chown -R "$USER:$USER" /var/lib/io.mikupush.server
chown -R "$USER:$USER" /var/log/io.mikupush.server
chown -R "$USER:$USER" /usr/share/io.mikupush.server

if [[ -n "$(which systemctl)" ]]; then
  systemctl daemon-reload
  systemctl enable mikupush-server.service
fi
