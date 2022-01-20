-- Add migration script here
CREATE TABLE channels (
	id TEXT
	,name TEXT
	,thumbnail TEXT
	,video_count INT
);

CREATE TABLE subscriptions (
	discord_id INT
	,channel_id TEXT
);