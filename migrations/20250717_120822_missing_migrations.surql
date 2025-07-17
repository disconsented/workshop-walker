DEFINE TABLE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD id ON votes TYPE { item: record<workshop_items>, link: record<properties> | record<companions>, user: record<users> } PERMISSIONS FULL;
DEFINE FIELD score ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD when ON votes TYPE datetime PERMISSIONS FULL;

REMOVE FIELD user ON votes;
REMOVE FIELD link ON votes;