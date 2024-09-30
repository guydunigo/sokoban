#!/bin/sh

cargo build
cat solution.txt | ./target/debug/sokoban
