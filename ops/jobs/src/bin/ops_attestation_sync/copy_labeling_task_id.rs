//! `copy_labeling_task_id` reads `acme.labeling.v1.LabelingTaskQueryService/LookupLabelingTask` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient::new(channel.clone());
    let fetched = source
        .lookup_labeling_task(labeling::LookupLabelingTaskRequest {
            context: Some(context.clone()),
            labeling_task_id: "live-copy-labeling-task-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let labeling_task = fetched.labeling_task.unwrap_or_default();
    let record = LabelingTaskIdRecord {
        labeling_task_id: labeling_task.labeling_task_id,
        subject: labeling_task.subject,
    };
    dispatch_labeling_task_id("feature-flag-admin", record, channel, &context).await
}

/// What `copy_labeling_task_id` keeps of each LookupLabelingTask response.
#[derive(Clone, Debug, Default)]
pub struct LabelingTaskIdRecord {
    pub labeling_task_id: String,
    pub subject: String,
}

/// Where `copy_labeling_task_id` hands each record.
trait LabelingTaskIdSink {
    fn deliver(&self, record: LabelingTaskIdRecord) -> Delivery;
}

/// Passes records to `acme.shows.v1.ShowCalculationService`.
struct ShowCalculationSink {
    client: shows::show_calculation_service_client::ShowCalculationServiceClient<Channel>,
}

impl LabelingTaskIdSink for ShowCalculationSink {
    fn deliver(&self, record: LabelingTaskIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_show(shows::CalculateShowRequest {
                    show_id: record.subject,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FeatureFlagAdminService`.
struct FeatureFlagAdminSink {
    client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient<Channel>,
}

impl LabelingTaskIdSink for FeatureFlagAdminSink {
    fn deliver(&self, record: LabelingTaskIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .archive_feature_flag(featureflags::ArchiveFeatureFlagRequest {
                    details: Some(featureflags::archive_feature_flag_request::Details {
                        reason_code: record.labeling_task_id,
                        ..Default::default()
                    }),
                    feature_flag_id: record.subject,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CsrValidationService`.
struct CsrValidationSink {
    client: certificates::csr_validation_service_client::CsrValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LabelingTaskIdSink for CsrValidationSink {
    fn deliver(&self, record: LabelingTaskIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::NormalizeCsrRequest {
                    reason: record.labeling_task_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.normalize_csr(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct LabelingTaskIdLogOnly;

impl LabelingTaskIdSink for LabelingTaskIdLogOnly {
    fn deliver(&self, record: LabelingTaskIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.labeling_task_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_labeling_task_id(
    kind: &str,
    record: LabelingTaskIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let show_calculation = ShowCalculationSink {
        client: shows::show_calculation_service_client::ShowCalculationServiceClient::new(channel.clone()),
    };
    let feature_flag_admin = FeatureFlagAdminSink {
        client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient::new(channel.clone()),
    };
    let csr_validation = CsrValidationSink {
        client: certificates::csr_validation_service_client::CsrValidationServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(LabelingTaskIdRecord) -> Delivery = match kind {
        "show-calculation" => &|record| show_calculation.deliver(record),
        "feature-flag-admin" => &|record| feature_flag_admin.deliver(record),
        "csr-validation" => &|record| csr_validation.deliver(record),
        _ => &|record| LabelingTaskIdLogOnly.deliver(record),
    };
    write(record).await
}
