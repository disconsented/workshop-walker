-- There seems to be _another_ bug where migrations break building an index.
REBUILD INDEX property_full_text_property ON properties;