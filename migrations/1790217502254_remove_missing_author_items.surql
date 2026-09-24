-- We weren't checking that items were "OK" when ingesting them, this has led to about 36 of them being invalid.
-- Its now patched, this just clears them out.
DELETE FROM workshop_items WHERE !author.exists();