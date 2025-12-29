
DEFINE FIELD OVERWRITE id ON votes TYPE {
	item: record<workshop_items>,
	link: record<properties> | record<companions>,
	user: record<users>
}
	PERMISSIONS FULL
;
DEFINE FIELD OVERWRITE score ON votes TYPE int
	PERMISSIONS FULL
;
DEFINE FIELD OVERWRITE when ON votes TYPE datetime
	PERMISSIONS FULL
;
DEFINE FIELD OVERWRITE user ON votes TYPE record<users>
	PERMISSIONS FULL
;
