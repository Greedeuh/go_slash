table! {
    global_features (features) {
        features -> Text,
    }
}

table! {
    shortcuts (shortcut, team_slug) {
        shortcut -> Text,
        team_slug -> Text,
        url -> Text,
    }
}

table! {
    teams (slug) {
        slug -> Text,
        title -> Text,
        is_private -> Bool,
        is_accepted -> Bool,
    }
}

table! {
    users (mail) {
        mail -> Text,
        pwd -> Text,
        is_admin -> Bool,
    }
}

table! {
    users_teams (user_mail, team_slug) {
        user_mail -> Text,
        team_slug -> Text,
        is_admin -> Bool,
        is_accepted -> Bool,
        rank -> SmallInt,
    }
}

joinable!(shortcuts -> teams (team_slug));
joinable!(users_teams -> teams (team_slug));
joinable!(users_teams -> users (user_mail));

allow_tables_to_appear_in_same_query!(global_features, shortcuts, teams, users, users_teams,);
