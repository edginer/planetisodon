-- MySQL (Planetscale)
CREATE TABLE IF NOT EXISTS boards (
    id INTEGER NOT NULL AUTO_INCREMENT,
    name TEXT NOT NULL,
    board_key TEXT NOT NULL,
    default_name VARCHAR(255) NOT NULL DEFAULT 'デフォルトの名無し',
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS threads (
    id VARCHAR(255) NOT NULL,
    thread_key INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    response_count INTEGER NOT NULL DEFAULT 1,
    ip_address TEXT NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_unix_timestamp INTEGER NOT NULL,
    author_id TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS responses (
    id VARCHAR(255) NOT NULL,
    thread_id VARCHAR(255) NOT NULL,
    name TEXT NOT NULL,
    mail TEXT NOT NULL,
    body TEXT NOT NULL,
    author_id TEXT NOT NULL,
    date_text TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(255) NOT NULL,
    ip_address TEXT NOT NULL,
    user_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    disabled INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

ALTER TABLE
    threads
ADD
    INDEX thread_key_index (thread_key);

ALTER TABLE
    threads
ADD
    INDEX board_id_index (board_id);

ALTER TABLE
    responses
ADD
    INDEX thread_id_index (thread_id);

ALTER TABLE
    users
ADD
    INDEX user_hash_index (user_hash);

-- Insert mock data into boards table
INSERT INTO
    boards (name, board_key)
VALUES
    ('EDGE-EXP', 'planetisodon');