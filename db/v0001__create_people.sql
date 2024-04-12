CREATE EXTENSION unaccent;

ALTER TEXT SEARCH DICTIONARY unaccent (RULES='unaccent');

CREATE TEXT SEARCH CONFIGURATION pessoas (COPY = portuguese);

ALTER TEXT SEARCH CONFIGURATION pessoas ALTER MAPPING FOR hword, hword_part, word WITH unaccent, portuguese_stem;

CREATE OR REPLACE FUNCTION ARRAY_TO_STRING_IMMUTABLE(
  arr TEXT[],
  sep TEXT
) RETURNS TEXT IMMUTABLE PARALLEL SAFE LANGUAGE SQL AS $$  SELECT ARRAY_TO_STRING(arr,sep) $$;

CREATE TABLE Pessoa (
  id UUID PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(100) UNIQUE NOT NULL,
  CPF VARCHAR(100) UNIQUE NOT NULL,
  balance INTEGER NOT NULL,
  tipo BOOLEAN NOT NULL,
  password VARCHAR(100) NOT NULL,
  search TSVECTOR GENERATED ALWAYS AS(
    TO_TSVECTOR('pessoas', name || ' ' || email || ' ' || CPF || ' ' || COALESCE(balance::TEXT, '') || ' ' || CASE WHEN tipo THEN 'true' ELSE 'false' END || ' ' || password)
  ) STORED,
  CONSTRAINT check_balance CHECK (balance >= 0) -- Assuming balance cannot be negative
);

CREATE INDEX pessoa_search_index ON Pessoa USING GIN(search);

CREATE TABLE Transacao (
  id UUID PRIMARY KEY NOT NULL,
  payee UUID REFERENCES Pessoa(id)NOT NULL,
  payer UUID REFERENCES Pessoa(id)NOT NULL,
  amount INTEGER NOT NULL,
  tempo DATE NOT NULL
);

