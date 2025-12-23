DEFINE TABLE usernames TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD id ON usernames TYPE record<users> PERMISSIONS FULL;
DEFINE FIELD last_updated ON usernames TYPE datetime PERMISSIONS FULL;
DEFINE FIELD name ON usernames TYPE string PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

DEFINE INDEX unique_id ON usernames FIELDS id;