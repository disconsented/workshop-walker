DEFINE TABLE OVERWRITE item_dependencies TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE in ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE out ON item_dependencies TYPE record<workshop_items> PERMISSIONS FULL;

DEFINE INDEX OVERWRITE dep_out ON item_dependencies FIELDS out;
