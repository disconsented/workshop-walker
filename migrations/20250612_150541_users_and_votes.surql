-- ------------------------------
-- TABLE: apps
-- ------------------------------

DEFINE TABLE OVERWRITE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

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

DEFINE FIELD OVERWRITE `value` ON properties TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE app_id ON properties TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE class ON properties TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE note ON properties TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source ON properties TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE upvote_count ON properties TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE vote_count ON properties TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE workshop_item ON properties TYPE record<workshop_items> PERMISSIONS FULL;



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

DEFINE FIELD OVERWRITE link ON votes TYPE record<properties> | record<companions> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE score ON votes TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE user ON votes TYPE record<users> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE when ON votes TYPE datetime PERMISSIONS FULL;



