#!/usr/bin/env bash

set -m

if ! command -v jq &> /dev/null
then
    echo "jq has to be installed to run this script"
    exit
fi

ignoredCrates="types\ndifferential_datalog\nrslint_scoping_ddlog"
packages=$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | .name')
cmd="$1"
shift

for package in $packages; do
  if !(echo "$ignoredCrates" | grep -Fq "$package"); then
    cargo $cmd -p $package $@
  fi
done
