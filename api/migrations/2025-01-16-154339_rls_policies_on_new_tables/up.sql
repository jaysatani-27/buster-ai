-- Your SQL goes here
ALTER TABLE dataset_groups ENABLE ROW LEVEL SECURITY;
ALTER TABLE dataset_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE datasets_to_dataset_groups ENABLE ROW LEVEL SECURITY;
ALTER TABLE datasets_to_permission_groups ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission_groups_to_users ENABLE ROW LEVEL SECURITY;
ALTER TABLE __diesel_schema_migrations ENABLE ROW LEVEL SECURITY;

CREATE POLICY dataset_groups_policy ON dataset_groups TO authenticated USING (true);
CREATE POLICY dataset_permissions_policy ON dataset_permissions TO authenticated USING (true);
CREATE POLICY datasets_to_dataset_groups_policy ON datasets_to_dataset_groups TO authenticated USING (true);
CREATE POLICY datasets_to_permission_groups_policy ON datasets_to_permission_groups TO authenticated USING (true);
CREATE POLICY permission_groups_to_users_policy ON permission_groups_to_users TO authenticated USING (true);
CREATE POLICY diesel_schema_migrations_policy ON __diesel_schema_migrations TO authenticated USING (true);
