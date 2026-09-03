-- Copy to new table
DEFINE TABLE workshop_item_properties_temp TYPE RELATION SCHEMALESS PERMISSIONS NONE;
FOR $record IN (
    SELECT *
    FROM workshop_item_properties
) {
    LET $in = $record.in;
    LET $out = $record.out;
    RELATE $in -> workshop_item_properties_temp -> $out SET
        note = $record.note,
        source = $record.source,
        status = $record.status,
        upvote_count = $record.upvote_count,
        vote_count = $record.vote_count;
};

-- Define new table
REMOVE TABLE workshop_item_properties;
DEFINE TABLE workshop_item_properties TYPE RELATION IN workshop_items OUT properties SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD note ON workshop_item_properties TYPE none | string PERMISSIONS FULL;
DEFINE FIELD source ON workshop_item_properties TYPE 'system' | record<users> PERMISSIONS FULL;
DEFINE FIELD status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;

DEFINE INDEX unique_workshop_item_properties ON workshop_item_properties FIELDS in, out UNIQUE;

-- Copy back and mutate

FOR $record IN (
    SELECT *
    FROM workshop_item_properties_temp
) {
    LET $in = type::record("workshop_items", type::number($record.in.id()));
    LET $out = $record.out;
    RELATE $in -> workshop_item_properties -> $out SET
        note = $record.note,
        source = $record.source,
        status = $record.status,
        upvote_count = $record.upvote_count,
        vote_count = $record.vote_count;
};

-- Drop old table
REMOVE TABLE workshop_item_properties_temp;
