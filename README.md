# Discord Voice Room Bot

This bot watches for folks to join a "watch"/"trigger" voice channel, e.g. "➕ Create a Room", and will create a voice channel for whoever joins it. The "room" channel will be deleted when no one is left in it.

## Deploying

We publish a container image for you to use. If one might want to deploy via Docker,

```sh
docker run -it \
  -e DISCORD_TOKEN=$MY_BOT_TOKEN \
  -e WATCH_CHANNEL_MY_SERVER=$SERVER_ID:$CHANNEL_ID \
  -e WATCH_CHANNEL_MY_OTHER_SERVER=$SERVER_ID_2:$CHANNEL_ID_2 \
    ghcr.io/planetside-community/discord-room-bot/bot:latest
```

You need a Discord Application (create one at https://discord.dev), and pass it's bot token in as `DISCORD_TOKEN`.

For each `WATCH_CHANNEL*` pattern environment variable, it must include which server to look at, and which channel is your "trigger channel", which must be a voice channel, separated by a `:`

If you're only using one WATCH_CHANNEL, you can omit the ending, and just have the variable named exactly that.

### Permissions

**This bot is really dumb. Only let it manage voice channels that you are willing to delete. It will refuse to delete the trigger channel, but all others it's happy to delete, it will delete any other empty ones.**

This typically means you'd want to give it Manage Channels access to one category.

### Setup

1. Create a category for your voice rooms.
2. Create a "trigger channel"
3. Copy your server ID and trigger channel ID, add it to your bot's environment variables in the form: `WATCH_CHANNEL=SERVER_ID:CHANNEL_ID`
4. Give the bot Manage Channels access to the category.
5. Join the trigger channel
6. You'll be placed in a room!
7. When you leave it, and it becomes empty, the room will disappear forever.
