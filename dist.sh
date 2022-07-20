#!/usr/bin/env bash

set -xe

_DIR=$(dirname $(realpath "$0"))

cd $_DIR

git add -u
git commit -m dist || true
git pull || true

cargo set-version --bump patch

cat Cargo.toml|grep version|head -1|awk -F \" '{print $2}' > .version

if ! hash mdi 2>/dev/null; then
cargo add mdi
fi

mdi

git add -u
git commit -m dist
git push

tag=v`cat ./.version`
git tag $tag
git push origin --tag $tag

cargo publish
