CREATE TABLE client
(
    id   SERIAL,
    uuid uuid NOT NULL,
    PRIMARY KEY ("id")
);

CREATE TABLE os_info
(
    client_id     integer,
    os            text,
    os_version    text,
    computer_name text NOT NULL,
    domain        text,
    PRIMARY KEY ("client_id"),
    CONSTRAINT "FK_os_info_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE software_info
(
    id        SERIAL,
    name      text NOT NULL,
    version   text NOT NULL,
    publisher text,
    PRIMARY KEY ("id")
);

CREATE TABLE software_list
(
    client_id   integer,
    software_id integer,
    PRIMARY KEY ("client_id", "software_id"),
    CONSTRAINT "FK_software_list_client" FOREIGN KEY ("client_id") REFERENCES "client" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT "FK_software_list_software_info" FOREIGN KEY ("software_id") REFERENCES "software_info" ("id") ON UPDATE CASCADE ON DELETE CASCADE
);