#!/bin/bash

if [[ -n "$(which systemctl)" ]]; then
  systemctl stop mikupush-server.service
  systemctl disable mikupush-server.service
fi
