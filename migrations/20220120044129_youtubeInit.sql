-- Add migration script here
create table users (
	guild_id INT
	,discord_id INT
);

CREATE TABLE channels (
	channel_id TEXT
	,title TEXT
	,thumbnail TEXT
	,video_count INT
);

CREATE TABLE subscriptions (
	discord_id INT
	,channel_id TEXT
);