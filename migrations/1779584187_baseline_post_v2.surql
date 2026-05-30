-- ------------------------------
-- TABLE: apps
-- ------------------------------

DEFINE TABLE OVERWRITE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE available ON apps TYPE bool DEFAULT true PERMISSIONS FULL;
DEFINE FIELD OVERWRITE banner ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE default_tags ON apps TYPE set<record<tags>> DEFAULT [] REFERENCE ON DELETE IGNORE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE default_tags.* ON apps TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE description ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE developer ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE enabled ON apps TYPE bool PERMISSIONS FULL;
DEFINE FIELD OVERWRITE id ON apps TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE name ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE tags ON apps TYPE set<record<tags>> DEFAULT [] REFERENCE ON DELETE IGNORE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE tags.* ON apps TYPE record<tags> PERMISSIONS FULL;



-- ------------------------------
-- TABLE: companions
-- ------------------------------

DEFINE TABLE OVERWRITE companions TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE id ON companions TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE in ON companions TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE note ON companions TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON companions TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source ON companions TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON companions TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE upvote_count ON companions TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE vote_count ON companions TYPE int DEFAULT 0 PERMISSIONS FULL;



-- ------------------------------
-- TABLE: item_dependencies
-- ------------------------------

DEFINE TABLE OVERWRITE item_dependencies TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE in ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;

DEFINE INDEX OVERWRITE dep_out ON item_dependencies FIELDS out;


-- ------------------------------
-- TABLE: properties
-- ------------------------------

DEFINE TABLE OVERWRITE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE id ON properties TYPE { class: string, value: string } PERMISSIONS FULL;


-- ------------------------------
-- TABLE: tags
-- ------------------------------

DEFINE TABLE OVERWRITE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE app_id ON tags TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE display_name ON tags TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE id ON tags TYPE string PERMISSIONS FULL;

DEFINE INDEX OVERWRITE field_app_id_tag ON tags FIELDS app_id, display_name;


-- ------------------------------
-- TABLE: usernames
-- ------------------------------

DEFINE TABLE OVERWRITE usernames TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE last_updated ON usernames TYPE datetime PERMISSIONS FULL;
DEFINE FIELD OVERWRITE name ON usernames TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE username ON usernames TYPE record<users> REFERENCE ON DELETE IGNORE PERMISSIONS FULL;

DEFINE INDEX OVERWRITE unique_username ON usernames FIELDS username;


-- ------------------------------
-- TABLE: users
-- ------------------------------

DEFINE TABLE OVERWRITE users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE admin ON users TYPE bool PERMISSIONS FULL;
DEFINE FIELD OVERWRITE banned ON users TYPE bool PERMISSIONS FULL;
DEFINE FIELD OVERWRITE id ON users TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE last_logged_in ON users TYPE datetime PERMISSIONS FULL;



-- ------------------------------
-- TABLE: votes
-- ------------------------------

DEFINE TABLE OVERWRITE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE id ON votes TYPE { item: record<workshop_items>, link: record<properties> | record<companions>, user: record<users> } PERMISSIONS FULL;
DEFINE FIELD OVERWRITE score ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE user ON votes TYPE record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE when ON votes TYPE datetime PERMISSIONS FULL;



-- ------------------------------
-- TABLE: workshop_item_properties
-- ------------------------------

DEFINE TABLE OVERWRITE workshop_item_properties TYPE RELATION IN workshop_items OUT properties SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE in ON workshop_item_properties TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE note ON workshop_item_properties TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON workshop_item_properties TYPE record<properties> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source ON workshop_item_properties TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;

DEFINE INDEX OVERWRITE unique_workshop_item_properties ON workshop_item_properties FIELDS in, out UNIQUE;


-- ------------------------------
-- TABLE: workshop_items
-- ------------------------------

DEFINE TABLE OVERWRITE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE appid ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE author ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE description ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE id ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE languages ON workshop_items TYPE set<int> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE languages.* ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE last_updated ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE preview_url ON workshop_items TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE score ON workshop_items TYPE float PERMISSIONS FULL;
DEFINE FIELD OVERWRITE tags ON workshop_items TYPE set<record<tags>> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE tags.* ON workshop_items TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE title ON workshop_items TYPE string PERMISSIONS FULL;

DEFINE INDEX OVERWRITE item_id ON workshop_items FIELDS id UNIQUE;
DEFINE INDEX OVERWRITE item_language ON workshop_items FIELDS languages;
DEFINE INDEX OVERWRITE item_score ON workshop_items FIELDS score;
DEFINE INDEX OVERWRITE item_title ON workshop_items FIELDS title;
DEFINE INDEX OVERWRITE item_updated ON workshop_items FIELDS last_updated;

-- Remove old surrealdb-migrations tables
REMOVE TABLE IF EXISTS migration;
REMOVE TABLE IF EXISTS migrations;
REMOVE TABLE IF EXISTS migration_lock;
REMOVE TABLE IF EXISTS migration_state;
REMOVE TABLE IF EXISTS __migrations;
REMOVE TABLE IF EXISTS script_migration;