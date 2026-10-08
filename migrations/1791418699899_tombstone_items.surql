DEFINE FIELD hidden ON workshop_items TYPE bool DEFAULT false PERMISSIONS FULL;
UPDATE workshop_items SET hidden = false;
DEFINE INDEX item_hidden ON workshop_items FIELDS hidden CONCURRENTLY;