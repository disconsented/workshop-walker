DEFINE TABLE OVERWRITE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD OVERWRITE id ON votes TYPE { item: record<workshop_items>, link: record<properties> | record<companions>, user: record<users> } PERMISSIONS FULL;
DEFINE FIELD OVERWRITE score ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE when ON votes TYPE datetime PERMISSIONS FULL;

REMOVE FIELD IF EXISTS user ON votes;
REMOVE FIELD IF EXISTS link ON votes;