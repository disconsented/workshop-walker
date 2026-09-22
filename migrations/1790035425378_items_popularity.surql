-- New fields:
-- subscriptions: int
-- lifetime_subscriptions: int
-- views: int
-- view_history: array
-- subscriptions_history: array
-- time_created: datetime

-- Pulled from steam API
DEFINE FIELD created ON workshop_items TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD lifetime_subscriptions ON workshop_items TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD subscriptions ON workshop_items TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD views ON workshop_items TYPE int DEFAULT 0 PERMISSIONS FULL;

-- Historical records
DEFINE FIELD view_history ON workshop_items TYPE array<int> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD subscription_history ON workshop_items TYPE array<int> DEFAULT [] PERMISSIONS FULL;

-- Calculated
DEFINE FIELD conversions ON workshop_items TYPE float DEFAULT subscriptions / views PERMISSIONS FULL;
DEFINE FIELD retention ON workshop_items TYPE float DEFAULT subscriptions / lifetime_subscriptions PERMISSIONS FULL;

UPDATE workshop_items SET conversions = 0.0, created = 0, lifetime_subscriptions = 0, retention = 0.0, subscription_history = [], subscriptions = 0, view_history = [], views = 0;
