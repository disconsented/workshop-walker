-- I'd realised that the best way to combat search bloat for the lookahead search was to
-- use the number of relations for a prop. But because surrealdb things you can't use a alias in a where clause :|

REMOVE INDEX workshop_item_property_full_text_property ON TABLE workshop_item_properties;
REMOVE FIELD prop_value ON workshop_item_properties;
UPDATE workshop_item_properties UNSET prop_value;

DEFINE FIELD value ON properties TYPE string VALUE id.id().value PERMISSIONS FULL;
UPDATE properties SET value = id.id().`value`;
DEFINE INDEX property_full_text_property ON TABLE properties FIELDS `value` FULLTEXT ANALYZER  property_analyzer;