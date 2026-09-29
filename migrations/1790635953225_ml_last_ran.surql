-- Running the ML on _everything_ takes way too damn long, and the results when I tried it sucked
-- So we're purging and tracking whether its been ran, the intent to run it "opportunistically"

DEFINE FIELD ml_last_run ON workshop_items TYPE int | NONE DEFAULT NONE PERMISSIONS FULL;
DELETE FROM workshop_item_properties WHERE out.id().class = 'Feature';
DELETE FROM properties WHERE id.id().class = 'Feature';