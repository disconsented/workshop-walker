-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: apps
-- ------------------------------

DEFINE TABLE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD available ON apps TYPE bool DEFAULT true PERMISSIONS FULL;
DEFINE FIELD banner ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD default_tags ON apps TYPE set<record<tags>> DEFAULT [] REFERENCE ON DELETE IGNORE PERMISSIONS FULL;
DEFINE FIELD default_tags.* ON apps TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD description ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD developer ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD enabled ON apps TYPE bool PERMISSIONS FULL;
DEFINE FIELD id ON apps TYPE int PERMISSIONS FULL;
DEFINE FIELD name ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD tags ON apps TYPE set<record<tags>> DEFAULT [] REFERENCE ON DELETE IGNORE PERMISSIONS FULL;
DEFINE FIELD tags.* ON apps TYPE record<tags> PERMISSIONS FULL;

DEFINE INDEX apps_id ON apps FIELDS id;


-- ------------------------------
-- TABLE: companions
-- ------------------------------

DEFINE TABLE companions TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON companions TYPE int PERMISSIONS FULL;
DEFINE FIELD in ON companions TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD note ON companions TYPE none | string PERMISSIONS FULL;
DEFINE FIELD out ON companions TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD source ON companions TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD status ON companions TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD upvote_count ON companions TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD vote_count ON companions TYPE int DEFAULT 0 PERMISSIONS FULL;



-- ------------------------------
-- TABLE: item_dependencies
-- ------------------------------

DEFINE TABLE item_dependencies TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD in ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD out ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;

DEFINE INDEX dep_out ON item_dependencies FIELDS out;


-- ------------------------------
-- TABLE: properties
-- ------------------------------

DEFINE TABLE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON properties TYPE { class: string, value: string } PERMISSIONS FULL;



-- ------------------------------
-- TABLE: script_migration
-- ------------------------------

DEFINE TABLE script_migration TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD executed_at ON script_migration TYPE datetime READONLY VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD script_name ON script_migration TYPE string PERMISSIONS FULL;



-- ------------------------------
-- TABLE: tags
-- ------------------------------

DEFINE TABLE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD app_id ON tags TYPE int PERMISSIONS FULL;
DEFINE FIELD display_name ON tags TYPE string PERMISSIONS FULL;
DEFINE FIELD id ON tags TYPE string PERMISSIONS FULL;

DEFINE INDEX field_app_id_tag ON tags FIELDS app_id, display_name;


-- ------------------------------
-- TABLE: usernames
-- ------------------------------

DEFINE TABLE usernames TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD last_updated ON usernames TYPE datetime PERMISSIONS FULL;
DEFINE FIELD name ON usernames TYPE string PERMISSIONS FULL;
DEFINE FIELD username ON usernames TYPE record<users> REFERENCE ON DELETE IGNORE PERMISSIONS FULL;

DEFINE INDEX unique_username ON usernames FIELDS username;


-- ------------------------------
-- TABLE: users
-- ------------------------------

DEFINE TABLE users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD admin ON users TYPE bool PERMISSIONS FULL;
DEFINE FIELD banned ON users TYPE bool PERMISSIONS FULL;
DEFINE FIELD id ON users TYPE string PERMISSIONS FULL;
DEFINE FIELD last_logged_in ON users TYPE datetime PERMISSIONS FULL;



-- ------------------------------
-- TABLE: votes
-- ------------------------------

DEFINE TABLE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON votes TYPE { item: record<workshop_items>, link: record<properties> | record<companions>, user: record<users> } PERMISSIONS FULL;
DEFINE FIELD score ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD user ON votes TYPE record<users> PERMISSIONS FULL;
DEFINE FIELD when ON votes TYPE datetime PERMISSIONS FULL;



-- ------------------------------
-- TABLE: workshop_item_properties
-- ------------------------------

DEFINE TABLE workshop_item_properties TYPE RELATION IN workshop_items OUT properties SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD in ON workshop_item_properties TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD note ON workshop_item_properties TYPE none | string PERMISSIONS FULL;
DEFINE FIELD out ON workshop_item_properties TYPE record<properties> PERMISSIONS FULL;
DEFINE FIELD source ON workshop_item_properties TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;

DEFINE INDEX unique_workshop_item_properties ON workshop_item_properties FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: workshop_items
-- ------------------------------

DEFINE TABLE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD appid ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD author ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD description ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD id ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD languages ON workshop_items TYPE set<int> PERMISSIONS FULL;
DEFINE FIELD languages.* ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD last_updated ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD preview_url ON workshop_items TYPE none | string PERMISSIONS FULL;
DEFINE FIELD score ON workshop_items TYPE float PERMISSIONS FULL;
DEFINE FIELD tags ON workshop_items TYPE set<record<tags>> PERMISSIONS FULL;
DEFINE FIELD tags.* ON workshop_items TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD title ON workshop_items TYPE string PERMISSIONS FULL;

DEFINE INDEX item_id ON workshop_items FIELDS id UNIQUE;
DEFINE INDEX item_language ON workshop_items FIELDS languages;
DEFINE INDEX item_score ON workshop_items FIELDS score;
DEFINE INDEX item_title ON workshop_items FIELDS title;
DEFINE INDEX item_updated ON workshop_items FIELDS last_updated;

-- Remove old surrealdb-migrations tables
REMOVE TABLE IF EXISTS migration;
REMOVE TABLE IF EXISTS migrations;
REMOVE TABLE IF EXISTS migration_lock;
REMOVE TABLE IF EXISTS migration_state;
REMOVE TABLE IF EXISTS __migrations;