-- More SurrealDB half assed testing
-- Set == BTreeSet, which, doesn't actually implement SurrealValue so you can't ser/de it...
-- Sorry but that's an asinine oversight

DEFINE FIELD OVERWRITE tags ON workshop_items TYPE array<record<tags>> VALUE $value.distinct() PERMISSIONS FULL;
UPDATE workshop_items SET tags = <array>tags;