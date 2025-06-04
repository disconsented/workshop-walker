DEFINE TABLE OVERWRITE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE appid ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE preview_url ON workshop_items TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE id ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE author ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE description ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE languages ON workshop_items TYPE set<int> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE last_updated ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD OVERWRITE tags ON workshop_items TYPE set<record<tags>> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE tags[*] ON workshop_items TYPE record<tags> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE title ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE score ON workshop_items TYPE float PERMISSIONS FULL;

DEFINE INDEX OVERWRITE item_updated ON workshop_items FIELDS last_updated;
DEFINE INDEX OVERWRITE item_language ON workshop_items FIELDS languages;
DEFINE INDEX OVERWRITE item_title ON workshop_items FIELDS title;
DEFINE INDEX OVERWRITE item_score ON workshop_items FIELDS score;
