#!/usr/bin/env bash

set -e

QBITTORRENT_WEB_API_VERSION="$(cat Cargo.toml | grep version | head -1 | sed 's/version = //' | sed 's/"//g')"
QBITTORRENT_WEB_API_GEN_VERSION="$(cat qbittorrent-web-api-gen/Cargo.toml | grep version | head -1 | sed 's/version = //' | sed 's/"//g')"

if [ $QBITTORRENT_WEB_API_VERSION != $QBITTORRENT_WEB_API_GEN_VERSION ]
then
  printf "Version mismatch, QBITTORRENT_WEB_API_VERSION=${QBITTORRENT_WEB_API_VERSION} and QBITTORRENT_WEB_API_GEN_VERSION=${QBITTORRENT_WEB_API_GEN_VERSION} (they must match)" >&2
  exit 1
fi
