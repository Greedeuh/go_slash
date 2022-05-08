table! {
    settings (title) {
        title -> Text,
        content -> Text,
    }
}

table! {
    shortcuts (shortcut, team_slug) {
        shortcut -> Varchar,
        team_slug -> Varchar,
        url -> Varchar,
    }
}

table! {
    teams (slug) {
        slug -> Varchar,
        title -> Varchar,
        is_private -> Bool,
        is_accepted -> Bool,
    }
}

table! {
    users (mail) {
        mail -> Varchar,
        pwd -> Nullable<Varchar>,
        capabilities -> Array<Text>,
    }
}

table! {
    users_teams (user_mail, team_slug) {
        user_mail -> Varchar,
        team_slug -> Varchar,
        capabilities -> Array<Text>,
        is_accepted -> Bool,
        rank -> Int2,
    }
}

joinable!(shortcuts -> teams (team_slug));
joinable!(users_teams -> teams (team_slug));
joinable!(users_teams -> users (user_mail));

allow_tables_to_appear_in_same_query!(
    settings,
    shortcuts,
    teams,
    users,
    users_teams,
);
