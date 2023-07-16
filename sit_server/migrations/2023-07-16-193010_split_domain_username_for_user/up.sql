ALTER TABLE "user" ADD "domain" TEXT NULL;
UPDATE "user" SET "username" = split_part("U"."username", '\', 2), "domain" = split_part("U"."username", '\', 1) FROM "user" AS "U";