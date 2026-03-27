#!/usr/bin/env -S just --justfile

# Make the default recipe just list possible recipes. Taken from the
# `just` documentation:
#     https://github.com/casey/just?tab=readme-ov-file#listing-available-recipes
# 
default:
    @just --list --unsorted --justfile {{justfile()}}

set dotenv-load := true

build-dir := 'target/'


alias b := build
alias c := clean
alias t := test
alias r := run
alias d := docs
alias od := open-docs


build:
    cargo build


clean:
    rm -r {{build-dir}}


test:
    cargo test


run *args:
    cargo run -- {{args}}


docs:
    cargo doc


open-docs: docs
    cargo doc --open
