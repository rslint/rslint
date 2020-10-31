#!/usr/bin/env bash

# ==============
# CI script to run the benchmark, and publish the results to criterion.dev
# ==============

HOST=https://api.criterion.dev

if [[ -z "$CRITERION_TOKEN" ]]; then
    echo "Must provide CRITERION_TOKEN in environment: export CRITERION_TOKEN=\"token\"" 1>&2
    exit 1
fi

GITHUB_USERNAME=$(echo $(git config --get remote.origin.url) | awk -F'/' '{print $4}')
GITHUB_REPO=$(echo $(git config --get remote.origin.url) | awk -F'/' '{print $5}')
GITHUB_REPO=$(echo ${GITHUB_REPO} | awk -F'.' '{print $1}')
GIT_COMMIT_HASH=$(echo $(git rev-parse HEAD))

cargo bench -p rslint_core -- --verbose --noplot --save-baseline criterion.dev.temp

UPLOAD_URL="$HOST/v1/$GITHUB_USERNAME/$GITHUB_REPO/measurements?token=$CRITERION_TOKEN&commit=$GIT_COMMIT_HASH"
UPLOAD_FILE_PATH=$(find $(find . -type d -name criterion.dev.temp) -name raw.csv)
UPLOAD_COMMAND="curl -F 'raw.csv=@$UPLOAD_FILE_PATH' '$UPLOAD_URL'"
eval "$UPLOAD_COMMAND"
