CREATE TABLE "auth_user" (
    "id" SERIAL,
    "username" TEXT NOT NULL,
    "password" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
CREATE TABLE "auth_sessions" (
    "id" SERIAL,
    "session_id" TEXT NOT NULL,
    "user_id" INTEGER NOT NULL,
    "valid_until" TIMESTAMP NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_auth_sessions_auth_user" FOREIGN KEY ("user_id") REFERENCES "auth_user" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);