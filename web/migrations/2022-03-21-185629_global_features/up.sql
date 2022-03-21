CREATE TABLE global_features (
  features text NOT NULL PRIMARY KEY
);

INSERT INTO global_features(features) VALUES ('{ "login": { "simple": false, "read_private": false, "write_private": false } }')
