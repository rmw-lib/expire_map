#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

UNAME=$( command -v uname)

case $( "${UNAME}" | tr '[:upper:]' '[:lower:]') in
  linux*)
    add(){
      sudo wget https://github.com/rmw-lib/file/raw/main/upx -O /usr/bin/upx
      sudo chmod +x /usr/bin/upx
    }
    ;;
  darwin*)
    add(){
      brew install $1
    }
    ;;
  msys*|cygwin*|mingw*|nt|win*)
    add(){
      choco install $1
    }
    ;;
  *)
    printf 'unknown\n'
    ;;
esac

if ! [ -x "$(command -v upx)" ]; then
add upx
fi
