#!/usr/bin/env bash

rm -r ../target
cargo build --release

previous_version="https://github.com/wllfaria/milky/releases/download/v0.1.0/milky_x86_64-unknown-linux-gnu_static.zip"
filename="milky-v0.1.0"

wget -O "$filename" "$previous_version"
unzip $filename
rm $filename
mv milky "$filename"

fastchess \
    -engine name=milky-v0.3.0 cmd=../target/release/milky \
    -engine name="$filename" cmd=./"$filename" \
    -each tc=1+0.1 \
    -maxmoves 100 \
    -rounds 300 \
    -recover \
    -concurrency 6 \
    -pgnout file=games.pgn \
    -openings file=./data/openings.epd format=epd \
    -sprt elo0=0 elo1=20 alpha=0.1 beta=0.1
