use snafu::{ResultExt, Whatever};
use tracing::trace;

use crate::{
    db::{
        IItemID, IUserID,
        model::{
            HistoryPair, InsertableWorkshopItem, InternalFullWorkshopItem, InternalWorkshopItem,
        },
    },
    domain::items::{History, ItemsError, ItemsPort},
    processing::language_actor::DetectedLanguage,
    steam::model::Child,
};

const HOTNESS_MODIFIER: f32 = 60.0 * 60.0 * 24.0 * 30.0 * 3.0;

pub struct ItemsService<R: ItemsPort> {
    repo: R,
}

impl<R: ItemsPort> ItemsService<R> {
    /// Queue ML on the following conditions:
    /// The model can support the language it's written in (currently just
    /// English) Either one of the following:
    /// 1) It's been accessed by a user (search/direct item) _and_ ml hasn't run
    ///    before
    /// 2) The description has changed
    pub async fn should_queue_ml(
        &self,
        item: &InternalWorkshopItem,
        activity_hint: bool,
    ) -> Result<bool, ItemsError> {
        let (existing_description, ml_last_ran) = self
            .repo
            .get_description_and_last_ml(item.id.clone())
            .await?;

        let description_changed = existing_description.as_ref() != Some(&item.description);
        // We don't want to waste our resources on extracting out of items that
        // the model won't support
        let viable_language = item.languages.contains(&DetectedLanguage::English);
        trace!(
            viable_language,
            description_changed,
            ?ml_last_ran,
            activity_hint,
            "decision"
        );
        Ok(viable_language && (activity_hint && ml_last_ran.is_none() || description_changed))
    }

    pub async fn get_item(&self, id: IItemID) -> Result<InternalWorkshopItem, ItemsError> {
        self.repo.get(id).await
    }

    pub async fn get_full_item(
        &self,
        id: IItemID,
        user: Option<IUserID>,
    ) -> Result<InternalFullWorkshopItem, ItemsError> {
        self.repo.get_full(id, user).await
    }

    pub async fn insert_data(
        &self,
        mut item: InternalWorkshopItem,
        children: Vec<Child>,
    ) -> crate::Result<(), Whatever> {
        let tags = std::mem::take(&mut item.tags);
        let id = item.id.clone();

        let history: Option<HistoryPair> = self
            .repo
            .get_history(id.clone())
            .await
            .whatever_context("getting history")?;

        // New items are missing all the things so nothing to query
        let history = history.unwrap_or_default();

        // Keep up to a year
        let view_history = History::from(history.view_history).push(item.views);

        let mut subscription_history =
            History::from(history.subscription_history).push(item.subscriptions);

        // Calculate trends
        let trend_week = subscription_history.calculate_wma(2 * 7);
        let trend_month = subscription_history.calculate_wma(2 * 30);
        let trend_quarter = subscription_history.calculate_wma(2 * 90);
        let trend_half = subscription_history.calculate_wma(2 * 180);
        let trend_year = subscription_history.calculate_wma(2 * 365);

        let subscription_history = subscription_history.into();

        let item = InsertableWorkshopItem {
            app: item.app,
            author: item.author.id,
            description: item.description,
            id: item.id,
            languages: item.languages,
            last_updated: item.last_updated,
            preview_url: item.preview_url,
            title: item.title,
            score: item.score,
            tags: tags.into_iter().map(|tag| tag.id).collect::<Vec<_>>(),
            lifetime_subscriptions: item.lifetime_subscriptions,
            subscriptions: item.subscriptions,
            views: item.views,
            view_history: view_history.into(),
            subscription_history,
            retention: item.retention,
            created: item.created,
            conversions: item.conversions,
            // Trying weight subscriptions against age, last_updated just raises huge mods back
            // up
            hotness: f32::log10(item.subscriptions.max(1) as f32)
                + item.created as f32 / HOTNESS_MODIFIER,
            trend_week,
            trend_month,
            trend_quarter,
            trend_half,
            trend_year,
        };
        self.repo
            .insert_data(item, children)
            .await
            .whatever_context("inserting item")
    }

    pub(crate) async fn update_ml_last_run(&self, id: IItemID) -> Result<(), ItemsError> {
        self.repo.update_ml_last_run(id).await
    }
}

impl<R: ItemsPort> ItemsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}
