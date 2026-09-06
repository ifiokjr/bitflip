BEGIN;

--
-- ACTION CREATE TABLE
--
CREATE TABLE "bitflip_colour_indexer_cursor" (
    "id" bigserial PRIMARY KEY,
    "cluster" text NOT NULL,
    "programAddress" text NOT NULL,
    "startSignature" text NOT NULL,
    "completedHeadSignature" text NOT NULL,
    "catchUpHeadSignature" text,
    "beforeSignature" text,
    "leaseToken" text,
    "leasedUntil" timestamp without time zone,
    "lastSuccessAt" timestamp without time zone,
    "updatedAt" timestamp without time zone NOT NULL
);

-- Indexes
CREATE UNIQUE INDEX "bitflip_colour_indexer_source_idx" ON "bitflip_colour_indexer_cursor" USING btree ("cluster", "programAddress");


--
-- MIGRATION VERSION FOR bitflip_server
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('bitflip_server', '20260906170748674-colour-indexer', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260906170748674-colour-indexer', "timestamp" = now();

--
-- MIGRATION VERSION FOR serverpod
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('serverpod', '20260824182259319', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260824182259319', "timestamp" = now();


COMMIT;
