#!/bin/bash
# sync_data.sh

RSYNC_PASS='geaccepteerd'
REMOTE_USER="voorwaarden"
REMOTE_HOST="data.ndovloket.nl"
REMOTE_PATH="/netex/"
LOCAL_PATH="./data/"

lftp -u voorwaarden,geaccepteerd sftp://data.ndovloket.nl -e "mirror --verbose --only-newer /netex data; quit"
