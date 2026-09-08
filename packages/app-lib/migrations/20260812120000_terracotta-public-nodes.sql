ALTER TABLE settings
ADD COLUMN terracotta_public_nodes JSONB NOT NULL DEFAULT '["wss://center.node.1tmc.top"]';
