CREATE TABLE "battery" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "battery_id" TEXT NOT NULL,
    "manufacturer" TEXT NOT NULL,
    "serial_number" TEXT NOT NULL,
    "chemistry" TEXT NOT NULL,
    "cycle_count" BIGINT NOT NULL,
    "designed_capacity" BIGINT NOT NULL,
    "full_charged_capacity" BIGINT NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_battery_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);