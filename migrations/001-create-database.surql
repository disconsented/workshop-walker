-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: item_dependencies
-- ------------------------------

DEFINE TABLE item_dependencies TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD in ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD out ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

DEFINE INDEX dep_out ON item_dependencies FIELDS out;



-- ------------------------------
-- TABLE DATA: item_dependencies
-- ------------------------------


-- ------------------------------
-- TABLE: tags
-- ------------------------------

DEFINE TABLE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD app_id ON tags TYPE int PERMISSIONS FULL;
DEFINE FIELD display_name ON tags TYPE string PERMISSIONS FULL;
DEFINE FIELD id ON tags TYPE string PERMISSIONS FULL;

DEFINE INDEX field_app_id_tag ON tags FIELDS app_id, tag;


-- ------------------------------
-- TABLE DATA: tags
-- ------------------------------


-- ------------------------------
-- TABLE: workshop_item
-- ------------------------------

DEFINE TABLE workshop_item TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD appid ON workshop_item TYPE int PERMISSIONS FULL;
DEFINE FIELD preview_url ON workshop_items TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD id ON workshop_item TYPE string PERMISSIONS FULL;
DEFINE FIELD language ON workshop_item TYPE string PERMISSIONS FULL;
DEFINE FIELD author ON workshop_item TYPE string PERMISSIONS FULL;
DEFINE FIELD language ON workshop_item TYPE string PERMISSIONS FULL;
DEFINE FIELD last_updated ON workshop_item TYPE int PERMISSIONS FULL;
DEFINE FIELD tags ON workshop_item TYPE set<record<tags>> PERMISSIONS FULL;
DEFINE FIELD tags[*] ON workshop_item TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD title ON workshop_item TYPE string PERMISSIONS FULL;

DEFINE INDEX item_updated ON tags FIELDS last_updated;
DEFINE INDEX item_title ON tags FIELDS title;

-- ------------------------------
-- TABLE DATA: workshop_item
-- ------------------------------


-- ------------------------------
-- TABLE: workshop_items
-- ------------------------------

DEFINE TABLE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD appid ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD description ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD id ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD title ON workshop_items TYPE string PERMISSIONS FULL;



-- ------------------------------
-- TABLE DATA: workshop_items
-- ------------------------------


-- ------------------------------
-- TABLE: workshop_tags
-- ------------------------------

DEFINE TABLE workshop_tags TYPE RELATION IN tags OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD in ON workshop_tags TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD out ON workshop_tags TYPE record<workshop_items> PERMISSIONS FULL;



-- ------------------------------
-- TABLE DATA: workshop_tags
-- ------------------------------


