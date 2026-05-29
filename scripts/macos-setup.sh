# This is setup for the macOS build process.
# Run this script once per command line session, and thereafter run 'cargo build'
# and 'cargo run' plain to compile and execute the project.

# This is needed for SDL2 linkage errors.

# This assumes you are using homebrew for your SDL2 installation.

export RUSTFLAGS="-L $(brew --prefix sdl2)/lib -l sdl2"
