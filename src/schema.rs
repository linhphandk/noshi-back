// @generated automatically by Diesel CLI.

diesel::table! {
    manual_platforms (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 50]
        platform -> Varchar,
        #[max_length = 255]
        handle -> Varchar,
        follower_count -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    password_resets (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        expires_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    profiles (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        slug -> Varchar,
        niches -> Array<Nullable<Text>>,
        #[max_length = 255]
        headline -> Varchar,
        is_published -> Bool,
        completion_score -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        refresh_token -> Varchar,
        expires_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    social_connections (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 50]
        platform -> Varchar,
        #[max_length = 255]
        platform_user_id -> Varchar,
        #[max_length = 255]
        handle -> Varchar,
        access_token_encrypted -> Bytea,
        refresh_token_encrypted -> Nullable<Bytea>,
        token_expires_at -> Timestamp,
        follower_count -> Int4,
        engagement_rate -> Nullable<Float8>,
        audience_demographics -> Nullable<Jsonb>,
        last_synced_at -> Nullable<Timestamp>,
        is_primary -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    waitlist (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        signed_up_at -> Timestamp,
    }
}

diesel::joinable!(manual_platforms -> users (user_id));
diesel::joinable!(password_resets -> users (user_id));
diesel::joinable!(profiles -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(social_connections -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    manual_platforms,
    password_resets,
    profiles,
    sessions,
    social_connections,
    users,
    waitlist,
);
