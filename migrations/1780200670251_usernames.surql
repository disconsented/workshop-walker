-- Migrate the IDs in usernames from string to int

-- This takes way too long, over an hour, cheaper to just download again
DELETE FROM usernames;
-- Copy to new table
DEFINE TABLE usernames_temp TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
INSERT INTO usernames_temp (SELECT *, id.id() AS id FROM usernames);

-- Define new table
REMOVE TABLE usernames;
DEFINE TABLE usernames TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON usernames TYPE int PERMISSIONS FULL;
DEFINE FIELD last_updated ON usernames TYPE datetime PERMISSIONS FULL;
DEFINE FIELD name ON usernames TYPE string PERMISSIONS FULL;

DEFINE INDEX unique_username ON usernames FIELDS id UNIQUE;
-- Copy back and mutate
LET $records = (SELECT * FROM usernames_temp);
FOR $record IN $records {
   LET $id = type::record("usernames", type::number($record.username.id()));
   UPSERT $id SET name = $record.name, last_updated = $record.last_updated;
};

-- Drop old table
REMOVE TABLE usernames_temp;