-- Add migration script here
create table players (
	discord_id integer
	,cash integer
	,daily_reset integer
	,packs text
	,packs_opened integer
	,packs_bought integer
	,total_cash integer
	,total_cards integer
	,cards_sold integer
	,daily_packs integer
	,quiz_questions integer
	,current_multiplier integer
	,quiz_correct integer
	,quiz_reset integer
	,savelist text
	,perm_multiplier integer
);

CREATE TABLE store (
	set_ids TEXT
	,daily_reset integer
);

create table packs (
	discord_id integer
	,set_id text
	,amount integer
);

create table cards (
	discord_id integer
	,card_id text
	,amount integer
);