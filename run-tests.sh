#!/bin/sh

echo "********** load from file         **********"
cargo run file
echo ""

echo "********** extract with zip       **********"
cargo run zip
echo ""

echo "********** extract hand-rolled    **********"
cargo run zip-parser
echo ""
