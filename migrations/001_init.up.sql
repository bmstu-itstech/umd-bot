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
    user_id    BIGINT      NOT NULL,

    PRIMARY KEY (slot_start, user_id),

    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES  users (id)
        ON DELETE CASCADE
);
