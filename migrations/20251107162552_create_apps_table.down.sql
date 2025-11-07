DROP TABLE IF EXISTS deployments_nodes;
DROP TABLE IF EXISTS deployments;
DROP TABLE IF EXISTS apps;
DROP TYPE IF EXISTS pin_status;
DROP TYPE IF EXISTS deployment_status;
DROP TRIGGER IF EXISTS set_deployments_nodes_updated_at ON deployments_nodes;
DROP TRIGGER IF EXISTS set_app_updated_at ON apps;
