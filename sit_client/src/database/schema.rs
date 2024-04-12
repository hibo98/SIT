// @generated automatically by Diesel CLI.

diesel::table! {
    client_task (id) {
        id -> Integer,
        task -> Text,
        time_start -> Nullable<BigInt>,
        time_download -> Nullable<BigInt>,
        task_status -> Integer,
        task_result -> Nullable<Text>,
    }
}
