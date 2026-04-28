CREATE TABLE "secure_boot" (
    "client_id" SERIAL NOT NULL,
    "available_updates" BIGINT NULL,
    "available_updates_policy" BIGINT NULL,
    "uefi_secure_boot_enabled" BIGINT NULL,
    "uefi_ca_2023_status" TEXT NULL,
    PRIMARY KEY ("client_id"),
    CONSTRAINT "FK_secure_boot_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);