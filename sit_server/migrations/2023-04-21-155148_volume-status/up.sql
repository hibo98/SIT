CREATE TABLE "volume_status" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "drive_letter" TEXT NOT NULL,
    "label" TEXT NULL,
    "file_system" TEXT NOT NULL,
    "capacity" NUMERIC(20, 0) NOT NULL,
    "free_space" NUMERIC(20,0) NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_volume_status_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
)