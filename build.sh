#!/usr/bin/env bash

set -ex

export PREFIX="$HOME/opt/cross"
export TARGET=i686-elf
export PATH="$PREFIX/bin:$PATH"

cargo build

rm -fr isodir
mkdir -p isodir/boot/grub
cp target/target/debug/kernel isodir/boot/myos.bin
cp grub.cfg isodir/boot/grub/grub.cfg
grub-mkrescue -o myos.iso isodir
