#!/bin/bash

if [[ -n "$(which systemctl)" ]]; then
  systemctl stop mikupush-server.service
fi
