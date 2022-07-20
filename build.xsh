#!/usr/bin/env xonsh

import tomli
from os.path import dirname,abspath,exists
from fire import Fire
import platform
from humanize import naturalsize
from os import stat,makedirs,replace
import tarfile

PWD = dirname(abspath(__file__))

cd @(PWD)

p".xonshrc".exists() && source .xonshrc


@Fire
def main():
  system = platform.system().lower()
  ext = ''

  if system == 'windows':
    os_name =  'win'
    machine = 'x86_64'
    os = f'{machine}-pc-windows-msvc'
    ext = '.exe'
  else:
    machine = platform.machine()

    if system == 'darwin':
      os_name =  'osx'
      os = f'{machine}-apple-{system}'
    elif system == 'linux':
      os_name =  'linux'
      os = f'{machine}-unknown-linux-gnu'

  $RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes"

# -l static=stdc++"

  TARGET=f'{os}'

  with open(join(PWD,"Cargo.toml"),"rb") as f:
    toml = tomli.load(f)

  pkg = toml['package']
  app = pkg['name']
  exe = app+ext
  version = pkg['version']

  cargo build \
  --release \
  --target @(TARGET) \
  -Z build-std=std,panic_abort \
  -Z build-std-features=panic_immediate_abort # 这一句有时候会导致问题，出问题了可以注释掉

  out=f"target/{TARGET}/release/{exe}"
  strip @(out)

  if system!='windows':
    ./sh/upx.sh

  upx --best --lzma @(out)

  print(naturalsize(stat(out).st_size))

  dir = 'target/txz'
  makedirs(join(PWD,dir),exist_ok=True)

  if machine == 'x86_64':
    machine = 'x64'

  txz = join(dir,app+f".{version}.{os_name}.{machine}.txz")
  with tarfile.open(txz, "w:xz") as tar:
    tar.add(out,arcname=exe)

  print(txz)
