// @generated automatically by Diesel CLI.

diesel::table! {
    waitlist (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        position -> Int4,
        profile_complete_at -> Nullable<Timestamp>,
        is_featured -> Bool,
        signed_up_at -> Timestamp,
    }
}
