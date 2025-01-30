ALTER TABLE "graphics_card"
    DROP CONSTRAINT "graphics_card_pkey",
    DROP COLUMN "id";
ALTER TABLE "graphics_card"
    ADD PRIMARY KEY ("client_id");