use std::collections::VecDeque;

use salvo::prelude::{StatusCode, StatusError};
use snafu::{ErrorCompat, prelude::*};

use crate::{
    db::{
        IItemID, IUserID,
        model::{
            HistoryPair, InsertableWorkshopItem, InternalFullWorkshopItem, InternalWorkshopItem,
        },
    },
    steam::model::Child,
};

const HISTORY_LIMIT: usize = 365 * 2;
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum ItemsError {
    #[snafu(whatever, display("{message}"))]
    Internal {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync + 'static>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },

    #[snafu(display("NotFound"))]
    NotFound,
}

impl ItemsError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ItemsError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ItemsError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<ItemsError> for StatusError {
    fn from(value: ItemsError) -> Self {
        let mut error = StatusError::internal_server_error();
        error.code = value.status_code();
        error.name = value
            .status_code()
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        error.brief = value.to_string();
        error.detail = value.backtrace().map(ToString::to_string);
        error
    }
}

/// Port for Item-related persistence operations.
pub trait ItemsPort: Send + Sync + 'static {
    async fn get(&self, id: IItemID) -> Result<InternalWorkshopItem, ItemsError>;
    async fn get_full(
        &self,
        id: IItemID,
        user: Option<IUserID>,
    ) -> Result<InternalFullWorkshopItem, ItemsError>;
    async fn get_description_and_last_ml(
        &self,
        id: IItemID,
    ) -> Result<(Option<String>, Option<u64>), ItemsError>;

    async fn get_history(&self, id: IItemID) -> Result<Option<HistoryPair>, ItemsError>;

    async fn insert_data(
        &self,
        item: InsertableWorkshopItem,
        children: Vec<Child>,
    ) -> Result<(), ItemsError>;

    async fn update_ml_last_run(&self, id: IItemID) -> Result<(), ItemsError>;
}

pub struct History(VecDeque<u64>);

impl History {
    pub(crate) fn push(mut self, item: u64) -> Self {
        while self.0.len() >= HISTORY_LIMIT {
            self.0.pop_front();
        }
        self.0.push_back(item);

        self
    }

    pub(crate) fn calculate_wma(&mut self, period: usize) -> f32 {
        // Whilst I'm still trying to figure out the _big swings in tiny mods_,
        // we'll just zero out the ones with not enough data. The hypothesis
        // being that relative, large growth will be peak early and be
        // squashed.
        if self.0.len() < period {
            // Tiny values, need send this to the shadow realm
            return f32::MIN;
        }

        let slice = self.0.make_contiguous();
        Self::calculate_relative_wma(&slice[slice.len().saturating_sub(period)..])
    }

    fn calculate_relative_wma(slice: &[u64]) -> f32 {
        let (weighted, total) = slice.windows(2).zip(1u32..).fold(
            (0.0f32, 0.0f32),
            |(weighted, total), (window, weight)| {
                let previous = window[0].max(1) as f32;
                let change = (window[1] as f32 - window[0] as f32) / previous;
                let weight = weight as f32;
                (weight.mul_add(change, weighted), total + weight)
            },
        );

        if total == 0.0 { 0.0 } else { weighted / total }
    }
}

impl From<Vec<u64>> for History {
    fn from(value: Vec<u64>) -> Self {
        Self(VecDeque::from(value))
    }
}

impl From<History> for Vec<u64> {
    fn from(value: History) -> Self {
        value.0.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::items::{HISTORY_LIMIT, History};

    #[test]
    fn test_bounds_wma() {
        let mut empty = History::from(vec![]);
        assert_eq!(empty.0.len(), 0);
        let _ = empty.calculate_wma(7 * 2);

        let mut partial = History::from(vec![]).push(1);
        assert_eq!(partial.0.len(), 1);
        let _ = partial.calculate_wma(7 * 2);

        let mut full = History::from(vec![0u64; HISTORY_LIMIT])
            .push(1)
            .push(1)
            .push(1);

        assert_eq!(full.0.len(), HISTORY_LIMIT);
        let _ = full.calculate_wma(7 * 2);
    }
}
