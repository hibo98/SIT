ALTER TABLE "graphics_card"
    ADD COLUMN "id" SERIAL,
    DROP CONSTRAINT "graphics_card_pkey";
ALTER TABLE "graphics_card"
    ADD PRIMARY KEY ("id");