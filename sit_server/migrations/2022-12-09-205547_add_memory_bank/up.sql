ALTER TABLE "memory_stick" ADD "bank_label" TEXT NOT NULL;
ALTER TABLE "memory_stick" ADD CONSTRAINT "unique_bank_label" UNIQUE ("client_id","bank_label");