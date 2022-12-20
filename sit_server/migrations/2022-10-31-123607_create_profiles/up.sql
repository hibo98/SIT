CREATE TABLE "user" (
    "id" SERIAL,
    "sid" TEXT NOT NULL UNIQUE,
    "username" TEXT NULL,
    PRIMARY KEY ("id")
);
CREATE TABLE "userprofile" (
    "client_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "health_status" SMALLINT NOT NULL,
    "roaming_configured" BOOLEAN NOT NULL,
    "roaming_path" TEXT NULL,
    "roaming_preference" BOOLEAN NULL,
    "last_use_time" TIMESTAMP NOT NULL,
    "last_download_time" TIMESTAMP NULL,
    "last_upload_time" TIMESTAMP NULL,
    "status" BIGINT NOT NULL,
    "size" NUMERIC(20, 0) NULL,
    PRIMARY KEY ("client_id", "user_id"),
    CONSTRAINT "FK_userprofile_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT "FK_userprofile_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
