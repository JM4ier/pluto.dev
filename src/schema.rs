table! {
    posts (url) {
        url -> Varchar,
        title -> Varchar,
        version -> Varchar,
        created -> Timestamp,
        updated -> Timestamp,
        content -> Text,
        published -> Nullable<Timestamp>,
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
        description -> Text,
    }
}

joinable!(tags -> tags_meta (tag));

allow_tables_to_appear_in_same_query!(
    posts,
    tags,
    tags_meta,
);
