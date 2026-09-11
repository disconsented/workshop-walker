DEFINE FIELD known_members ON tags TYPE int DEFAULT 0 PERMISSIONS FULL;
UPDATE tags SET known_members = 0;