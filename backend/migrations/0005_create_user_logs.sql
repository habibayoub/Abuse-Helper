CREATE TABLE user_logs (
    uuid UUID PRIMARY KEY,
    user_uuid UUID REFERENCES users(uuid),
    action VARCHAR(255) NOT NULL,
    route VARCHAR(255) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_logs_user_uuid ON user_logs(user_uuid);
CREATE INDEX idx_user_logs_timestamp ON user_logs(timestamp);
