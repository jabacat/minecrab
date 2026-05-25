# Minecrab

## About

This is a Minecraft clone made in Rust.

## Goals

The goals of this project are to practice "traditional" programming, no AI, and
collaboration with other developers. To that end ...

## Rules

No AI. So that's no vibecoding, generating code using LLMs generally, asking LLMs questions, no autocomplete, etc.
This project is for us to practice our development skills and build something cool all by ourselves.

## Build Instructions

### macOS

Assuming you have installed GLFW via Homebrew, run `source scripts/macos-setup.sh`
one per command linesession to get the correct Rust compilation flags.

## Assets

Our main texture atlas is 320x320 px and contains all of our textures. Each time it is changed, we increment the
version number and commit both the source xcf and png files. The textures start on the bottom left and more will
be added later.
