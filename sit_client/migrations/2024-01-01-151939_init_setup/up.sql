CREATE TABLE client_task (
    id              INTEGER PRIMARY KEY NOT NULL,
    task            TEXT NOT NULL,
    time_start      BIGINT NULL,
    time_download   BIGINT NULL,
    task_status     INTEGER NOT NULL,
    task_result     TEXT NULL
);
