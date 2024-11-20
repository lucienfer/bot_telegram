-- Add migration script here
CREATE TABLE public.user (
  "user_id" int8 NOT NULL,
  "chat_id" int8 NOT NULL,
  "addr" text NOT NULL,
  "notification" BOOL NOT NULL
);