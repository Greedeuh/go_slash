CREATE TABLE teams (
  slug        VARCHAR NOT NULL PRIMARY KEY,
  title       VARCHAR NOT NULL,
  is_private  BOOLEAN NOT NULL,
  is_accepted BOOLEAN NOT NULL
);

CREATE TABLE users_teams (
  user_mail   VARCHAR NOT NULL,
  team_slug   VARCHAR NOT NULL,
  is_admin    BOOLEAN NOT NULL,
  is_accepted BOOLEAN NOT NULL,
  FOREIGN KEY (user_mail) REFERENCES users(mail),
  FOREIGN KEY (team_slug) REFERENCES teams(slug),
  PRIMARY KEY (user_mail, team_slug)
);

ALTER TABLE shortcuts 
  ADD team_slug VARCHAR NOT NULL REFERENCES teams(slug);

ALTER TABLE users 
  ADD is_admin BOOLEAN NOT NULL;