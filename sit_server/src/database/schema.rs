// @generated automatically by Diesel CLI.

diesel::table! {
    bios (client_id) {
        client_id -> Int4,
        name -> Text,
        manufacturer -> Text,
        version -> Text,
    }
}

diesel::table! {
    client (id) {
        id -> Int4,
        uuid -> Uuid,
    }
}

diesel::table! {
    computer_model (client_id) {
        client_id -> Int4,
        manufacturer -> Text,
        model_family -> Text,
        serial_number -> Text,
    }
}

diesel::table! {
    disks (id) {
        id -> Int4,
        client_id -> Int4,
        model -> Text,
        serial_number -> Text,
        size -> Nullable<Numeric>,
        device_id -> Text,
        status -> Text,
        media_type -> Text,
    }
}

diesel::table! {
    graphics_card (client_id) {
        client_id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    license_key (id) {
        id -> Int4,
        client_id -> Int4,
        name -> Text,
        key -> Text,
    }
}

diesel::table! {
    memory_stick (id) {
        id -> Int4,
        client_id -> Int4,
        capacity -> Nullable<Numeric>,
        bank_label -> Text,
    }
}

diesel::table! {
    network_adapter (id) {
        id -> Int4,
        client_id -> Int4,
        name -> Text,
        mac_address -> Nullable<Text>,
    }
}

diesel::table! {
    network_adapter_ip (id) {
        id -> Int4,
        adapter_id -> Int4,
        ip -> Text,
    }
}

diesel::table! {
    os_info (client_id) {
        client_id -> Int4,
        os -> Nullable<Text>,
        os_version -> Nullable<Text>,
        computer_name -> Text,
        domain -> Nullable<Text>,
    }
}

diesel::table! {
    processor (client_id) {
        client_id -> Int4,
        name -> Text,
        manufacturer -> Text,
        cores -> Int8,
        logical_cores -> Int8,
        clock_speed -> Int8,
        address_width -> Int4,
    }
}

diesel::table! {
    software_info (id) {
        id -> Int4,
        name -> Text,
        publisher -> Nullable<Text>,
    }
}

diesel::table! {
    software_list (client_id, software_id) {
        client_id -> Int4,
        software_id -> Int4,
    }
}

diesel::table! {
    software_version (id) {
        id -> Int4,
        software_id -> Int4,
        version -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Int4,
        sid -> Text,
        username -> Nullable<Text>,
    }
}

diesel::table! {
    userprofile (client_id, user_id) {
        client_id -> Int4,
        user_id -> Int4,
        health_status -> Int2,
        roaming_configured -> Bool,
        roaming_path -> Nullable<Text>,
        roaming_preference -> Nullable<Bool>,
        last_use_time -> Timestamp,
        last_download_time -> Nullable<Timestamp>,
        last_upload_time -> Nullable<Timestamp>,
        status -> Int8,
        size -> Nullable<Numeric>,
    }
}

diesel::table! {
    userprofile_paths (id) {
        id -> Int4,
        client_id -> Int4,
        user_id -> Int4,
        path -> Text,
        size -> Numeric,
    }
}

diesel::table! {
    volume_status (id) {
        id -> Int4,
        client_id -> Int4,
        drive_letter -> Text,
        label -> Nullable<Text>,
        file_system -> Text,
        capacity -> Numeric,
        free_space -> Numeric,
    }
}

diesel::joinable!(bios -> client (client_id));
diesel::joinable!(computer_model -> client (client_id));
diesel::joinable!(disks -> client (client_id));
diesel::joinable!(graphics_card -> client (client_id));
diesel::joinable!(license_key -> client (client_id));
diesel::joinable!(memory_stick -> client (client_id));
diesel::joinable!(network_adapter -> client (client_id));
diesel::joinable!(network_adapter_ip -> network_adapter (adapter_id));
diesel::joinable!(os_info -> client (client_id));
diesel::joinable!(processor -> client (client_id));
diesel::joinable!(software_list -> client (client_id));
diesel::joinable!(software_list -> software_version (software_id));
diesel::joinable!(software_version -> software_info (software_id));
diesel::joinable!(userprofile -> client (client_id));
diesel::joinable!(userprofile -> user (user_id));
diesel::joinable!(userprofile_paths -> client (client_id));
diesel::joinable!(userprofile_paths -> user (user_id));
diesel::joinable!(volume_status -> client (client_id));

diesel::allow_tables_to_appear_in_same_query!(
    bios,
    client,
    computer_model,
    disks,
    graphics_card,
    license_key,
    memory_stick,
    network_adapter,
    network_adapter_ip,
    os_info,
    processor,
    software_info,
    software_list,
    software_version,
    user,
    userprofile,
    userprofile_paths,
    volume_status,
);
