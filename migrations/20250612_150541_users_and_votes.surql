-- ------------------------------
-- TABLE: apps
-- ------------------------------

DEFINE TABLE OVERWRITE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE id ON apps TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE available ON apps TYPE bool PERMISSIONS FULL;
DEFINE FIELD OVERWRITE banner ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE default_tags ON apps TYPE set<record<tags>> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE default_tags[*] ON apps TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE description ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE developer ON apps TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE enabled ON apps TYPE bool PERMISSIONS FULL;
DEFINE FIELD OVERWRITE name ON apps TYPE string PERMISSIONS FULL;



-- ------------------------------
-- TABLE: companions
-- ------------------------------

DEFINE TABLE OVERWRITE companions TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE id ON companions TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE in ON companions TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE note ON companions TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON companions TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source ON companions TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON companions TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE upvote_count ON companions TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE vote_count ON companions TYPE int DEFAULT 0 PERMISSIONS FULL;




-- ------------------------------
-- TABLE: properties
-- ------------------------------

DEFINE TABLE OVERWRITE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON properties TYPE { class: string, value: string } PERMISSIONS FULL;



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

DEFINE FIELD OVERWRITE id ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE link ON votes TYPE record<properties> | record<companions> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE score ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE user ON votes TYPE record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE when ON votes TYPE datetime PERMISSIONS FULL;

-- ------------------------------
-- TABLE: workshop_item_properties
-- ------------------------------

DEFINE TABLE OVERWRITE workshop_item_properties TYPE RELATION IN workshop_items OUT properties SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE in ON workshop_item_properties TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE note ON workshop_item_properties TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON workshop_item_properties TYPE record<properties> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source ON workshop_item_properties TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;

DEFINE INDEX unique_workshop_item_properties ON workshop_item_properties FIELDS in, out UNIQUE;
