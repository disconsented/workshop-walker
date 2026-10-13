-- It ends up being this index nukes performance. From 75ms to 1.5s in testing.
-- WITH INDEX item_updated - 75ms
-- WITH NOINDEX - 140ms
-- WITH INDEX item_hidden - 1.5s

REMOVE INDEX item_hidden ON workshop_items;