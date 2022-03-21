table! {
    global_features (features) {
        features -> Text,
    }
}

table! {
    shortcuts (shortcut) {
        shortcut -> Text,
        url -> Text,
    }
}

table! {
    users (mail) {
        mail -> Text,
        pwd -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    global_features,
    shortcuts,
    users,
);
