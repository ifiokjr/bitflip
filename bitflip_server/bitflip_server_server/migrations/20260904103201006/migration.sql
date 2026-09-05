BEGIN;

--
-- Function: gen_random_uuid_v7()
-- Source: https://gist.github.com/kjmph/5bd772b2c2df145aa645b837da7eca74
-- License: MIT (copyright notice included on the generator source code).
--
create or replace function gen_random_uuid_v7()
returns uuid
as $$
begin
  -- use random v4 uuid as starting point (which has the same variant we need)
  -- then overlay timestamp
  -- then set version 7 by flipping the 2 and 1 bit in the version 4 string
  return encode(
    set_bit(
      set_bit(
        overlay(uuid_send(gen_random_uuid())
                placing substring(int8send(floor(extract(epoch from clock_timestamp()) * 1000)::bigint) from 3)
                from 1 for 6
        ),
        52, 1
      ),
      53, 1
    ),
    'hex')::uuid;
end
$$
language plpgsql
volatile;

--
-- ACTION CREATE TABLE
--
CREATE TABLE "bitflip_mint_challenge" (
    "id" uuid PRIMARY KEY DEFAULT gen_random_uuid_v7(),
    "walletAddress" text NOT NULL,
    "gameIndex" bigint NOT NULL,
    "sectionIndex" bigint NOT NULL,
    "nonce" text NOT NULL,
    "message" text NOT NULL,
    "createdAt" timestamp without time zone NOT NULL,
    "expiresAt" timestamp without time zone NOT NULL,
    "usedAt" timestamp without time zone
);

-- Indexes
CREATE UNIQUE INDEX "bitflip_mint_challenge_nonce_idx" ON "bitflip_mint_challenge" USING btree ("nonce");
CREATE INDEX "bitflip_mint_challenge_wallet_created_idx" ON "bitflip_mint_challenge" USING btree ("walletAddress", "createdAt");
CREATE INDEX "bitflip_mint_challenge_expiry_idx" ON "bitflip_mint_challenge" USING btree ("expiresAt");


--
-- MIGRATION VERSION FOR bitflip_server
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('bitflip_server', '20260904103201006', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260904103201006', "timestamp" = now();

--
-- MIGRATION VERSION FOR serverpod
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('serverpod', '20260824182259319', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260824182259319', "timestamp" = now();


COMMIT;
