-- Add migration script here
create table users (
	guild_id BIGINT
	,discord_id BIGINT
);

CREATE TABLE channels (
	channel_id TEXT
	,title TEXT
	,thumbnail TEXT
	,video_count INT
);

CREATE TABLE subscriptions (
	discord_id BIGINT
	,channel_id TEXT
);