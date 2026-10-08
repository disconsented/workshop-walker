-- Clear all the indexes just in case
REBUILD INDEX property_full_text_property ON properties CONCURRENTLY;
REBUILD INDEX unique_workshop_item_properties ON workshop_item_properties CONCURRENTLY;
