# Salt-Discordbot
The open source code behind the Salt discord's awesome faucet bot!
This bot is implemented in 100% Rust, using [`twilight`](https://docs.rs/twilight/latest/twilight/) as the discord interaction layer.

## Deployment process / Debugging
The discordbot currently runs on a Salt-owned private (digital ocean) server.
You need an SSH key to gain access, and to be on the company VPN.

## Killing previously running session
To stop the bot from running, try running the `/admin kill` discord slash command in the test server.

## On the server
The process runs using `pm2`:
```nushell
pm2 ls
pm2 logs
```
The binary is located at `/home/ah/Desktop/rust-discordbot`.
Logs are saved by day at `/home/ah/Desktop/logs`.
These paths are hard-coded when building this project for release, so no environment variables are needed
to properly start the bot on the server side.