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
