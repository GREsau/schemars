#!/bin/bash
set -euxo pipefail

cd schemars/examples

rm -f *.schema.json

for file in *.rs
do
  example=${file%.rs}
  cargo run --example "$example" > "$example.schema.json"
done

cd ../..

rm -f docs/_includes/examples/*.rs
rm -f docs/_includes/examples/*.schema.json

cp schemars/examples/* docs/_includes/examples/
