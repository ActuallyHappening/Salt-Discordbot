# Salt-Discordbot

## Deploy
```nushell
# Building locally
cargo build --release

# Copying binary to server however you like
# to the path /home/ah/Desktop/rust-discordbot
scp target/release/salt-discordbot salt:///home/ah/Desktop/rust-discordbot

# Run the bot after its on the server
# Note: probably better to use pm2, pueue or something
ssh salt /home/ah/Desktop/rust-discordbot
```

## Private APIs
A private crate exists as a git submodule in this repo.

```nushell
git submodule init

# to update
git submodule update --remote
```
