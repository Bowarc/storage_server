cargo watch -s "clear && sh ./scripts/clean_back.sh & sh ./scripts/build_back.sh && cargo r -p back" -w ./back -w ./Rocket.toml --why
