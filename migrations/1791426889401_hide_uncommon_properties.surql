-- New properties from the ML are "hidden" (-2) until they occur more than once, then they're accepted directly.
-- Should cut down on the one offs.
DEFINE FIELD OVERWRITE status ON workshop_item_properties TYPE -2 | -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
-- Hide all the one offs
LET $groups = (
    SELECT
        id,
        out,
        in.*.app as app
    FROM workshop_item_properties
    WHERE
        source = "system"
        AND status == 1
    GROUP BY out, app
).filter(
    |$property_group| {
        $property_group.id.len() == 1;
    }
);
FOR $group IN $groups {
    FOR $id IN {
        $group.id;
    } {
        UPDATE $id SET status = -2;
    };
};