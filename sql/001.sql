DROP TABLE IF EXISTS account CASCADE;
DROP TABLE IF EXISTS category CASCADE;
DROP TABLE IF EXISTS entry CASCADE;
DROP TABLE IF EXISTS clearing CASCADE;

CREATE TABLE account
(
    id           UUID PRIMARY KEY,
    name         TEXT    NOT NULL,
    clear_amount NUMERIC NOT NULL DEFAULT (0)
);
INSERT INTO account (id, name)
VALUES ('78ed4192-03d7-4d60-83d0-e4f530a64417', 'test');

CREATE TABLE category
(
    id   UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE clearing
(
    id      UUID PRIMARY KEY,
    account UUID      NOT NULL REFERENCES account (id),
    amount  NUMERIC,
    stamp   TIMESTAMP NOT NULL
);

CREATE TABLE entry
(
    id       UUID PRIMARY KEY,
    account  UUID      NOT NULL REFERENCES account (id),
    category UUID REFERENCES category (id),
    stamp    TIMESTAMP NOT NULL,
    memo     TEXT      NOT NULL DEFAULT (''),
    amount   NUMERIC,
    cleared  UUID REFERENCES clearing (id)
);

CREATE OR REPLACE PROCEDURE new_entry(
    p_id UUID,
    p_account UUID
)
    LANGUAGE plpgsql
AS
$$
BEGIN
    INSERT INTO entry (id, account, stamp)
    VALUES (p_id, p_account, now());
END;
$$;

CREATE OR REPLACE FUNCTION get_balance(
    p_account UUID
)
    RETURNS NUMERIC
AS
$$
DECLARE
    last_cleared NUMERIC;
    uncleared    NUMERIC;
BEGIN
    SELECT amount
    INTO last_cleared
    FROM clearing
    WHERE account = p_account
    ORDER BY stamp DESC
    LIMIT 1;

    SELECT SUM(amount)
    INTO uncleared
    FROM entry
    WHERE account = p_account
      AND cleared IS NULL;

    RETURN COALESCE(last_cleared, 0) + COALESCE(uncleared, 0);
END;
$$ LANGUAGE plpgsql;
