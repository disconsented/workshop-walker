use std::fmt::{Display, Formatter};

use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Default, PartialEq)]
pub enum OrderBy {
    Hotness,
    Subscriptions,
    TrendWeek,
    #[default]
    TrendMonth,
    TrendQuarter,
    TrendHalf,
    TrendYear,
    LastUpdated,
    Rating,
    Dependents,
}

impl OrderBy {
    pub fn column_name(&self) -> &str {
        match self {
            OrderBy::LastUpdated => "last_updated",
            OrderBy::Hotness => "hotness",
            OrderBy::Subscriptions => "subscriptions",
            OrderBy::TrendWeek => "trend_week",
            OrderBy::TrendMonth => "trend_month",
            OrderBy::TrendQuarter => "trend_quarter",
            OrderBy::TrendHalf => "trend_half",
            OrderBy::TrendYear => "trend_year",
            OrderBy::Rating => "score",
            OrderBy::Dependents => "dependents",
        }
    }
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
