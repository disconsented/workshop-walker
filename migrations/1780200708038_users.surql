-- Migrate the IDs in users from string to int
-- Copy to new table
DEFINE TABLE users_temp TYPE NORMAL SCHEMALESS PERMISSIONS NONE;
INSERT INTO users_temp (SELECT *, id.id() AS id FROM users);

-- Define new table
REMOVE TABLE users;
DEFINE TABLE users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD id ON users TYPE int PERMISSIONS FULL;
DEFINE FIELD admin ON users TYPE bool PERMISSIONS FULL;
DEFINE FIELD banned ON users TYPE bool PERMISSIONS FULL;
DEFINE FIELD last_logged_in ON users TYPE datetime PERMISSIONS FULL;

-- Copy back and mutate
INSERT INTO users (SELECT *, type::number(id.id()) AS id FROM users_temp);
-- Drop old table
REMOVE TABLE users_temp;