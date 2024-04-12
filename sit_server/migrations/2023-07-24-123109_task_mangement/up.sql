CREATE TYPE "task_status" AS ENUM ('created', 'downloaded', 'running', 'successful', 'failed');
CREATE TABLE "client_task" (
	"id" SERIAL,
	"client_id" INTEGER NOT NULL,
	"task" JSON NOT NULL,
	"time_start" TIMESTAMP NULL,
	"time_download" TIMESTAMP NULL,
    "task_status" task_status,
	"task_result" JSON NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "FK_client_task_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);
