CREATE SEQUENCE time_split_pk;
CREATE TABLE time_split(
    id INTEGER PRIMARY KEY DEFAULT nextval('time_split_pk'),
    name VARCHAR NOT NULL,
    description VARCHAR,
    deleted BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE time_split_timer(
    time_split_id INTEGER NOT NULL REFERENCES time_split(id),
    len INTERVAL NOT NULL,
    name VARCHAR NOT NULL,
    work BOOLEAN NOT NULL
)

CREATE SEQUENCE timesheet_group_pk;
CREATE TABLE timesheet_group(
    timesheet_group BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('timesheet_group_pk'),
    time_split_id INTEGER NOT NULL REFERENCES time_split(id)
);

CREATE TABLE timesheet(
    timesheet_group BIGINT NOT NULL REFERENCES timesheet_group(timesheet_group),
    start_time TIMESTAMP_MS NOT NULL PRIMARY KEY,
    end_time TIMESTAMP_MS NOT NULL UNIQUE,
    work BOOLEAN NOT NULL
);

CREATE SEQUENCE tag_pk;
CREATE TABLE tag(
    id INTEGER PRIMARY KEY DEFAULT nextval('tag_pk'),
    tag VARCHAR NOT NULL UNIQUE,
    deleted BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE timesheet_tag(
    timesheet_group BIGINT NOT NULL REFERENCES timesheet_group(timesheet_group),
    tag_id INTEGER NOT NULL REFERENCES tag(id),
    PRIMARY KEY (timesheet_group, tag_id)
);

INSERT INTO time_split (id, name) VALUES (0, '_paused_');
INSERT INTO time_split (name, description) VALUES
    ('Pomodoro', 'Classic, tried, and true'),
    ('Time Magazine', 'Based on studies'),
    ('Tyson Split', 'For those with extra dog in ''em'),
    ('Build Night', 'We burnin'' out tonight baby!');

INSERT INTO time_split_timer (time_split_id, len, name, work) VALUES
    -- _paused_
    (0, INTERVAL 0 MINUTES, '_paused_', false),
    -- Pomodoro
    (1, INTERVAL 25 MINUTES, 'Work', true),
    (1, INTERVAL 5 MINUTES, 'Break', false),
    (1, INTERVAL 25 MINUTES, 'Work', true),
    (1, INTERVAL 15 MINUTES, 'Long Break', false),
    -- Time Magazine
    (2, INTERVAL 52 MINUTES, 'Work', true),
    (2, INTERVAL 17 MINUTES, 'Break', false),
    -- Tyson Split
    (3, INTERVAL 90 MINUTES, 'Work', true),
    (3, INTERVAL 10 MINUTES, 'Break', false),
    -- Build Night
    (4, INTERVAL 120 MINUTES, 'Work', true),
    (4, INTERVAL 10 MINUTES, 'Break', false);
