-- Yeah I know this is a bit excessive but there are refinements to the prompts/processes that should make the output better

-- Clear props, votes & prop links
DELETE FROM workshop_item_properties;
DELETE FROM properties;
DELETE FROM votes;

-- Clear all the indexes just in case
REBUILD INDEX property_full_text_property ON properties CONCURRENTLY;
REBUILD INDEX unique_workshop_item_properties ON workshop_item_properties CONCURRENTLY;

UPDATE workshop_items SET ml_last_run = NONE;