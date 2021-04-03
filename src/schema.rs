table! {
    posts (path) {
        path -> Varchar,
        title -> Varchar,
        version -> Varchar,
        published -> Bool,
        created -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}
