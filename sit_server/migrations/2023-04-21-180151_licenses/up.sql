CREATE TABLE "license_key" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_license_key_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
