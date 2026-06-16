-- More SurrealDB half assed testing
-- Set == BTreeSet, which, doesn't actually implement SurrealValue so you can't ser/de it...
-- Sorry but that's an asinine oversight

DEFINE FIELD OVERWRITE tags ON apps TYPE array<record<tags>> VALUE $value.distinct() PERMISSIONS FULL;
DEFINE FIELD OVERWRITE default_tags ON apps TYPE array<record<tags>> VALUE $value.distinct() PERMISSIONS FULL;
UPDATE apps SET tags = <array>tags;
UPDATE apps SET default_tags = <array>default_tags;