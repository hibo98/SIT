CREATE TABLE "userprofile_paths" (
    "id" SERIAL,
    "client_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "path" TEXT NOT NULL,
    "size" NUMERIC(20, 0) NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_userprofile_paths_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT "FK_userprofile_paths_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
CREATE INDEX "INDEX_userprofile_paths_client_id" ON "userprofile_paths" ("client_id");
CREATE INDEX "INDEX_userprofile_paths_user_id" ON "userprofile_paths" ("user_id");
