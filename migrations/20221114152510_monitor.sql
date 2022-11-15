CREATE TABLE IF NOT EXISTS monitor (
  channel_id text NOT NULL,
  user_id char(24) NOT NULL,
  username text NOT NULL,
  last_match_id char(24),
  game_time double precision NOT NULL,
  last_personal_best_blitz integer,
  last_personal_best_40l integer,
  PRIMARY KEY(channel_id, user_id)
);