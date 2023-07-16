UPDATE "user" SET "username" = "U"."username" || '\' || "U"."domain" FROM "user" AS "U";
ALTER TABLE "user" DROP "domain" TEXT NULL;