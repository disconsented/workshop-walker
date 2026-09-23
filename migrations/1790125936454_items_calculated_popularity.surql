-- Trend: Week
-- Trend: Month
-- Trend: Quarter
-- Trend: Half
-- Trend: Year

-- Hot: log10(max(subs,1)) + time_created / 60*60*24*31*3(quarter)

DEFINE FIELD trend_week ON workshop_items TYPE float DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD trend_month ON workshop_items TYPE float DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD trend_quarter ON workshop_items TYPE float DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD trend_half ON workshop_items TYPE float DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD trend_year ON workshop_items TYPE float DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD hotness ON workshop_items TYPE float DEFAULT 0 PERMISSIONS FULL;

UPDATE workshop_items SET trend_week = 0.0, trend_month = 0.0, trend_quarter = 0.0, trend_half = 0.0, trend_year = 0.0, hotness = 0.0;

DEFINE INDEX item_trend_week ON workshop_items FIELDS trend_week;
DEFINE INDEX item_trend_month ON workshop_items FIELDS trend_month;
DEFINE INDEX item_trend_quarter ON workshop_items FIELDS trend_quarter;
DEFINE INDEX item_trend_half ON workshop_items FIELDS trend_half;
DEFINE INDEX item_trend_year ON workshop_items FIELDS trend_year;
DEFINE INDEX item_hotness ON workshop_items FIELDS hotness;
DEFINE INDEX item_subscriptions ON workshop_items FIELDS subscriptions;
