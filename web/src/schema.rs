table! {
    global_features (features) {
        features -> Text,
    }
}

table! {
    shortcuts (shortcut) {
        shortcut -> Text,
        url -> Text,
        team_slug -> Nullable<Text>,
    }
}

table! {
    teams (slug) {
        slug -> Text,
        title -> Text,
    }
}

table! {
    users (mail) {
        mail -> Text,
        pwd -> Text,
    }
}

table! {
    users_teams (user_mail, team_slug) {
        user_mail -> Text,
        team_slug -> Text,
        accepted -> Bool,
    }
}

joinable!(shortcuts -> teams (team_slug));
joinable!(users_teams -> teams (team_slug));
joinable!(users_teams -> users (user_mail));

allow_tables_to_appear_in_same_query!(
    global_features,
    shortcuts,
    teams,
    users,
    users_teams,
);
