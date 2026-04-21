-- This should only be ran after a manual migration from v2 to v3

-- Migrate from appid int to app_id record
DEFINE FIELD app_id ON workshop_items TYPE record<apps> PERMISSIONS FULL;
UPDATE workshop_items SET app_id = type::record("apps", appid), languages = <set> languages, tags = <set> tags;
REMOVE FIELD appid ON workshop_items;