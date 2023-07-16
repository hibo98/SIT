UPDATE "user" AS "u1" SET "u1"."username" = "u2"."username" || '\' || "u2"."domain" FROM "user" AS "u2" WHERE "u1"."id" = "u2"."id";
ALTER TABLE "user" DROP "domain";