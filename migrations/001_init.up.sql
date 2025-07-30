DO $$ BEGIN
    CREATE TYPE SERVICE AS ENUM (
        'initial_registration', 
        'visa',
        'renewal_of_registration',
        'renewal_of_visa',
        'all'
    );
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE users (
    id                BIGINT      PRIMARY KEY,
    username          VARCHAR(32) UNIQUE NOT NULL,
    full_name_lat     VARCHAR     NOT NULL,
    full_name_cyr     VARCHAR     NOT NULL,
    citizenship       VARCHAR(32) NOT NULL,
    arrival_date      DATE        NOT NULL
);

CREATE TABLE reservations (
    slot_start TIMESTAMPTZ NOT NULL,
    service    SERVICE     NOT NULL,
    user_id    BIGINT      NOT NULL,

    PRIMARY KEY (slot_start, user_id),

    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES  users (id)
        ON DELETE CASCADE
);
