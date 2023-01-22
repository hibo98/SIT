CREATE TABLE "software_version" (
    "id" SERIAL,
    "software_id" INTEGER NOT NULL,
    "version" TEXT NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_software_version_software_info" FOREIGN KEY ("software_id") REFERENCES "software_info" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
INSERT INTO "software_version" ("software_id","version") SELECT "software_info"."id", "software_info"."version" FROM "software_info";
ALTER TABLE "software_list" DROP CONSTRAINT "FK_software_list_software_info";
ALTER TABLE "software_list" DROP CONSTRAINT "software_list_pkey";
UPDATE "software_list" SET ("software_id") = (SELECT "software_version"."id" FROM "software_version" LEFT JOIN "software_info" ON "software_info"."id" = "software_version"."software_id" WHERE "software_info"."id" = "software_list"."software_id" LIMIT 1);
ALTER TABLE "software_list" ADD CONSTRAINT "FK_software_list_software_version" FOREIGN KEY ("software_id") REFERENCES "software_version" ("id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE "software_list" ADD PRIMARY KEY ("client_id", "software_id");
ALTER TABLE "software_info" DROP "version";
