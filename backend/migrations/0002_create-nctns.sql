CREATE TABLE IF NOT EXISTS nctns (
    uuid UUID PRIMARY KEY,
    source_time VARCHAR(255),
    time TIMESTAMP WITH TIME ZONE,
    ip VARCHAR(45),
    reverse_dns VARCHAR(255),
    domain_name VARCHAR(255),
    asn VARCHAR(255),
    as_name VARCHAR(255),
    category VARCHAR(255),
    type VARCHAR(255),
    malware_family VARCHAR(255),
    vulnerability VARCHAR(255),
    tag VARCHAR(255),
    source_name VARCHAR(255),
    comment TEXT,
    description TEXT,
    description_url VARCHAR(255),
    destination_ip VARCHAR(45),
    destination_port INTEGER,
    port INTEGER,
    protocol VARCHAR(255),
    transport_protocol VARCHAR(255),
    http_request TEXT,
    user_agent TEXT,
    username VARCHAR(255),
    url TEXT,
    destination_domain_name VARCHAR(255),
    status VARCHAR(255),
    observation_time TIMESTAMP WITH TIME ZONE,
    source_feed VARCHAR(255)
);

INSERT INTO nctns (uuid, source_time, time, ip, reverse_dns, domain_name, asn, as_name, category, type, malware_family, vulnerability, tag, source_name, comment, description, description_url, destination_ip, destination_port, port, protocol, transport_protocol, http_request, user_agent, username, url, destination_domain_name, status, observation_time, source_feed)
VALUES
(gen_random_uuid(), 'source1', '2024-03-18 12:00:00', '192.168.1.1', 'example.com', 'example.com', 'AS12345', 'Example AS', 'Category1', 'Type1', 'MalwareFamily1', 'Vulnerability1', 'Tag1', 'SourceName1', 'Comment1', 'Description1', 'http://example.com/description1', '192.168.2.1', 80, 8080, 'TCP', 'HTTP', 'GET /page1', 'Mozilla/5.0', 'user1', 'http://example.com/page1', 'example.com', 'Success', '2024-03-18 12:01:00', 'feed1'),
(gen_random_uuid(), 'source2', '2024-03-18 13:00:00', '192.168.1.2', 'example.net', 'example.net', 'AS54321', 'Example AS', 'Category2', 'Type2', 'MalwareFamily2', 'Vulnerability2', 'Tag2', 'SourceName2', 'Comment2', 'Description2', 'http://example.com/description2', '192.168.2.2', 443, 8443, 'TCP', 'HTTPS', 'POST /page2', 'Chrome', 'user2', 'http://example.com/page2', 'example.net', 'Failed', '2024-03-18 13:01:00', 'feed2');
