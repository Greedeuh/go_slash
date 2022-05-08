CREATE TABLE teams (
  slug        VARCHAR NOT NULL PRIMARY KEY,
  title       VARCHAR NOT NULL,
  is_private  BOOLEAN NOT NULL,
  is_accepted BOOLEAN NOT NULL
);

INSERT INTO teams (slug, title, is_private, is_accepted) VALUES ('', 'Global', false, true);

CREATE TABLE shortcuts (
  shortcut  VARCHAR NOT NULL,
  team_slug VARCHAR NOT NULL,
  url       VARCHAR NOT NULL,
  FOREIGN KEY (team_slug) REFERENCES teams(slug) ON DELETE CASCADE,
  PRIMARY KEY (shortcut, team_slug)
);

CREATE TABLE users (
  mail          VARCHAR NOT NULL PRIMARY KEY,
  pwd           VARCHAR,
  capabilities  text[]  NOT NULL DEFAULT ARRAY[]::text[]
); 

CREATE TABLE settings (
  title text NOT NULL PRIMARY KEY,
  content text NOT NULL
);

INSERT INTO settings(title, content) VALUES
  ('default_capabilities', '[]'),
  ('features', '{ "login": { "simple": false, "google": false,"read_private": false }, "teams": false }');

CREATE TABLE users_teams (
  user_mail   VARCHAR NOT NULL,
  team_slug   VARCHAR NOT NULL,
  capabilities  text[]  NOT NULL DEFAULT ARRAY[]::text[],
  is_accepted BOOLEAN NOT NULL,
  rank        SMALLINT NOT NULL,
  FOREIGN KEY (user_mail) REFERENCES users(mail) ON DELETE CASCADE,
  FOREIGN KEY (team_slug) REFERENCES teams(slug) ON DELETE CASCADE,
  PRIMARY KEY (user_mail, team_slug)
);