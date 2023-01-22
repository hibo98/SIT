ALTER TABLE "software_list" DROP CONSTRAINT "FK_software_list_software_version";
DROP TABLE "software_version";
ALTER TABLE "software_list" ADD CONSTRAINT "FK_software_list_software_info" FOREIGN KEY ("software_id") REFERENCES "software_info" ("id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE "software_info" ADD "version" TEXT NOT NULL;