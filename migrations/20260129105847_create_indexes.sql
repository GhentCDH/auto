-- For searching by name across main entities
CREATE INDEX idx_application_name ON application(name);
CREATE INDEX idx_service_name ON service(name);
CREATE INDEX idx_infra_name ON infra(name);
CREATE INDEX idx_domain_name ON domain(name);
CREATE INDEX idx_person_name ON person(name);
CREATE INDEX idx_stack_name ON stack(name);
CREATE INDEX idx_network_share_name ON network_share(name);

-- For filtering on environment
CREATE INDEX idx_application_environment ON application(environment);
CREATE INDEX idx_service_environment ON service(environment);
