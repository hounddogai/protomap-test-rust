//! Persistence boundary for featureflags. The audit payload is never used as an RPC request.

use acme_platform::journal::{Journal, StoreError};
use acme_proto::acme::common::v1::RequestContext;
use prost::Message;

#[derive(Clone)]
pub struct FeatureflagsRepository {
    journal: Journal,
}

impl FeatureflagsRepository {
    pub fn new(journal: Journal) -> Self {
        Self { journal }
    }

    pub async fn record<T: Message>(
        &self,
        method: &str,
        context: &RequestContext,
        record: &T,
    ) -> Result<(), StoreError> {
        self.journal
            .append("featureflags", method, &context.tenant_id, record.encode_to_vec())
            .await
    }
}
