use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use twinned::dual_struct;

use crate::db::{IUsernameID, UsernameID};

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct Username {
    #[dual_type(IUsernameID)]
    pub id: UsernameID,
    pub name: String,
}

pub(crate) fn to_external_username(
    internal: Option<InternalUsername>,
) -> Result<Option<ExternalUsername>, surrealdb_types::Error> {
    internal.map(TryInto::try_into).transpose()
}

pub(crate) fn to_internal_username(external: Option<ExternalUsername>) -> Option<InternalUsername> {
    external.map(Into::into)
}
