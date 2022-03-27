
ALTER TABLE shortcuts 
  DROP team_slug;

ALTER TABLE users 
  DROP is_admin;

DROP TABLE users_teams;
DROP TABLE teams;