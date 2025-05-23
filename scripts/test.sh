#!/usr/bin/env bash

rm -r ../target
cargo build --release

previous_version="https://github.com/wllfaria/milky/releases/download/v0.1.0/milky_x86_64-unknown-linux-gnu_static.zip"
filename="milky-v0.1.0-1981elo"

wget -O "$filename" "$previous_version"
unzip $filename
rm $filename
mv milky "$filename"

fastchess \
    -engine name=milky-v0.1.1 cmd=../target/release/milky \
    -engine name=milky-v0.1.0-1981elo cmd=./"$filename" \
    -each tc=1+0.1 \
    -maxmoves 100 \
    -rounds 2000 \
    -recover \
    -concurrency 6 \
    -pgnout file=games.pgn \
    -openings file=./openings.epd format=epd order=random\
