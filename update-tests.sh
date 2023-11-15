#!/bin/bash
set -euxo pipefail

cd schemars

rm -f tests/actual/*.json

TRYBUILD=overwrite cargo test --all-features --no-fail-fast --tests || :

if ls tests/actual/*.json 1> /dev/null 2>&1; then
  mv -f tests/actual/*.json tests/expected/
else
  echo "Test schemas are up-to-date."
fi
