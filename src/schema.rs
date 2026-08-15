// @generated automatically by Diesel CLI.

diesel::table! {
    waitlist (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        signed_up_at -> Timestamp,
    }
}
