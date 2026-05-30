-- More SurrealDB half assed testing
-- Set == BTreeSet, which, doesn't actually implement SurrealValue so you can't ser/de it...
-- Sorry but that's an asinine oversite

DEFINE FIELD OVERWRITE languages ON workshop_items TYPE array<int> VALUE $value.distinct() PERMISSIONS FULL;
UPDATE workshop_items SET languages = <array>languages;