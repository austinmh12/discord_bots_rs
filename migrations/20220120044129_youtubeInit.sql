-- Add migration script here
create table users (
	guild_id TEXT NOT NULL
	,discord_id TEXT NOT NULL
);

CREATE TABLE channels (
	channel_id TEXT NOT NULL
	,title TEXT NOT NULL
	,thumbnail TEXT NOT NULL
	,video_count INT NOT NULL DEFAULT 0
);

CREATE TABLE subscriptions (
	discord_id TEXT NOT NULL
	,channel_id TEXT NOT NULL
);