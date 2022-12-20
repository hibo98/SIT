CREATE TABLE "computer_model" (
    "client_id" INTEGER NOT NULL,
    "manufacturer" TEXT NOT NULL,
    "model_family" TEXT NOT NULL,
    "serial_number" TEXT NOT NULL,
    PRIMARY KEY ("client_id"),
    CONSTRAINT "FK_computer_model_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "memory_stick" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "capacity" NUMERIC(20, 0) NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_memory_stick_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "processor" (
    "client_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "manufacturer" TEXT NOT NULL,
    "cores" BIGINT NOT NULL,
    "logical_cores" BIGINT NOT NULL,
    "clock_speed" BIGINT NOT NULL,
    "address_with" INTEGER NOT NULL,
    PRIMARY KEY ("client_id"),
    CONSTRAINT "FK_processor_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "disks" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "model" TEXT NOT NULL,
    "serial_number" TEXT NOT NULL,
    "size" NUMERIC(20, 0) NULL,
    "device_id" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "media_type" TEXT NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_disks_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "network_adapter" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "mac_address" TEXT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_network_adapter_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "network_adapter_ip" (
    "id" SERIAL,
    "adapter_id" INTEGER NOT NULL,
    "ip" TEXT NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_network_adapter_ip_adapter_id" FOREIGN KEY ("adapter_id") REFERENCES "network_adapter" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "graphics_card" (
    "client_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    PRIMARY KEY ("client_id"),
    CONSTRAINT "FK_graphics_card_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE TABLE "bios" (
    "client_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "manufacturer" TEXT NOT NULL,
    "version" TEXT NOT NULL,
    PRIMARY KEY ("client_id"),
    CONSTRAINT "FK_bios_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
