CREATE TABLE nctns (
    uuid VARCHAR(255) PRIMARY KEY,
    source_time VARCHAR(255),
    "time" TIMESTAMP,
    ip VARCHAR(255),
    reverse_dns VARCHAR(255),
    domain_name VARCHAR(255),
    asn VARCHAR(255),
    as_name VARCHAR(255),
    category VARCHAR(255),
    "type" VARCHAR(255),
    malware_family VARCHAR(255),
    vulnerability VARCHAR(255),
    tag VARCHAR(255),
    source_name VARCHAR(255),
    comment TEXT,
    "description" TEXT,
    description_url VARCHAR(255),
    destination_ip VARCHAR(255),
    destination_port INTEGER,
    port INTEGER,
    protocol VARCHAR(255),
    transport_protocol VARCHAR(255),
    http_request TEXT,
    user_agent TEXT,
    username VARCHAR(255),
    "url" TEXT,
    destination_domain_name VARCHAR(255),
    "status" VARCHAR(255),
    observation_time TIMESTAMP,
    source_feed VARCHAR(255)
);
