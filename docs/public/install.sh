#!/bin/sh
set -e
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/frectonz/penny/releases/latest/download/penny-installer.sh | sh
