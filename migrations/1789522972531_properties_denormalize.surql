-- Can't index record links, so we're denormaizing

DEFINE FIELD prop_value ON workshop_item_properties TYPE string VALUE out.id().value PERMISSIONS FULL;
UPDATE workshop_item_properties SET prop_value = out.id().value;