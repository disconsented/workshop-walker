-- Migrate the IDs in item dependencies from strings to integers

-- Copy to new table
DEFINE TABLE item_dependencies_temp TYPE RELATION SCHEMALESS PERMISSIONS NONE;
FOR $record IN (
    SELECT *
    FROM item_dependencies
) {
    LET $in = $record.in;
    LET $out = $record.out;
    RELATE $in -> item_dependencies_temp -> $out;
};

-- Define new table
REMOVE TABLE item_dependencies;
DEFINE TABLE item_dependencies TYPE RELATION IN workshop_items OUT workshop_items SCHEMAFULL PERMISSIONS NONE;

DEFINE INDEX dep_out ON item_dependencies FIELDS out;

-- Copy back and mutate

FOR $record IN (
    SELECT *
    FROM item_dependencies_temp
) {
    LET $in = type::record("workshop_items", type::number($record.in.id()));
    LET $out = type::record("workshop_items", type::number($record.out.id()));
    RELATE $in -> item_dependencies -> $out;
};

-- Drop old table
REMOVE TABLE item_dependencies_temp;
