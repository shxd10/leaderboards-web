CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE cup (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE track (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cup_id INTEGER NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,

    FOREIGN KEY (cup_id) REFERENCES cup(id)
);


CREATE TABLE category (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE record (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    track_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    flap BOOLEAN NOT NULL DEFAULT 0,
    lap1 INTEGER,
    lap2 INTEGER,
    lap3 INTEGER,
    time_ms INTEGER NOT NULL,
    proof TEXT,
    submitted_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES track(id),
    FOREIGN KEY (category_id) REFERENCES category(id),

    UNIQUE (user_id, track_id, category_id)
);
