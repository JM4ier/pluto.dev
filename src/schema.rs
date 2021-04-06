table! {
    posts (url) {
        url -> Varchar,
        title -> Varchar,
        version -> Varchar,
        published -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
        content -> Text,
    }
}

table! {
    tags (tag, url) {
        tag -> Varchar,
        url -> Varchar,
    }
}

table! {
    tags_meta (tag) {
        tag -> Varchar,
        display -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    posts,
    tags,
    tags_meta,
);
