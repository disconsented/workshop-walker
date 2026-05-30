-- Migrate workshop_items
-- id's type has been changed from a string to an int
-- app has been renamed from appid & changed to a record
-- author has been changed to a record

-- Copy to new table
DEFINE TABLE workshop_items_temp TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
INSERT INTO workshop_items_temp (SELECT *, id.id() AS id FROM workshop_items);

-- Define new table
REMOVE TABLE workshop_items;
DEFINE TABLE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD app ON workshop_items TYPE record<apps> PERMISSIONS FULL;
DEFINE FIELD author ON workshop_items TYPE record<usernames> PERMISSIONS FULL;
DEFINE FIELD description ON workshop_items TYPE string PERMISSIONS FULL;
DEFINE FIELD languages ON workshop_items TYPE set<int> PERMISSIONS FULL;
DEFINE FIELD last_updated ON workshop_items TYPE int PERMISSIONS FULL;
DEFINE FIELD preview_url ON workshop_items TYPE none | string PERMISSIONS FULL;
DEFINE FIELD score ON workshop_items TYPE float PERMISSIONS FULL;
DEFINE FIELD tags ON workshop_items TYPE set<record<tags>> PERMISSIONS FULL;
DEFINE FIELD title ON workshop_items TYPE string PERMISSIONS FULL;

DEFINE INDEX item_id ON workshop_items FIELDS id UNIQUE;
DEFINE INDEX item_language ON workshop_items FIELDS languages;
DEFINE INDEX item_score ON workshop_items FIELDS score;
DEFINE INDEX item_title ON workshop_items FIELDS title;
DEFINE INDEX item_updated ON workshop_items FIELDS last_updated;

-- Copy back and mutate
INSERT INTO workshop_items (
    SELECT *, 
    type::number(id.id()) AS id, 
    type::record('usernames', type::number(author)) as author, 
    type::record('apps', type::number(appid)) as app,
    type::set(languages) as languages,
    type::set(tags) as tags
 OMIT appid
 FROM workshop_items_temp);

-- Drop old table
REMOVE TABLE workshop_items_temp;