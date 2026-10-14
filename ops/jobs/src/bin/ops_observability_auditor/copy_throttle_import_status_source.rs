//! `copy_throttle_import_status_source` reads
//! `acme.ratelimiting.v1.ThrottleImportService/GetThrottleImportStatus` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::throttle_import_service_client::ThrottleImportServiceClient::new(channel.clone());
    let fetched = source
        .get_throttle_import_status(ratelimiting::GetThrottleImportStatusRequest {
            context: Some(context.clone()),
            throttle_id: "main-copy-throttle-import-status-source".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = fetched.throttle.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = ThrottleImportStatusSourceRecord {
        source: throttle.source,
        status: result.status,
        correlation_id: throttle.correlation_id,
        kind: acme_kit::text::normalize(&throttle.kind),
    };
    let mut sinks: BTreeMap<&str, Box<dyn ThrottleImportStatusSourceSink>> = BTreeMap::new();
    sinks.insert(
        "warehouse",
        Box::new(WarehouseSink {
            client: warehousing::warehouse_service_client::WarehouseServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "notification",
        Box::new(NotificationSink {
            client: notifications::notification_service_client::NotificationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "secret-workflow",
        Box::new(SecretWorkflowSink {
            client: secrets::secret_workflow_service_client::SecretWorkflowServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "throttle-reporting",
        Box::new(ThrottleReportingSink {
            client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("warehouse") else {
        anyhow::bail!("no sink registered for warehouse");
    };
    sink.write(record).await
}

/// What `copy_throttle_import_status_source` keeps of each GetThrottleImportStatus response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleImportStatusSourceRecord {
    pub source: String,
    pub status: String,
    pub correlation_id: String,
    pub kind: String,
}

/// Where `copy_throttle_import_status_source` hands each record.
trait ThrottleImportStatusSourceSink {
    fn write(&self, record: ThrottleImportStatusSourceRecord) -> Delivery;
}

/// Writes records to `acme.warehousing.v1.WarehouseService`.
struct WarehouseSink {
    client: warehousing::warehouse_service_client::WarehouseServiceClient<Channel>,
}

impl ThrottleImportStatusSourceSink for WarehouseSink {
    fn write(&self, record: ThrottleImportStatusSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_warehouse(warehousing::GetWarehouseRequest {
                    warehouse_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.notifications.v1.NotificationService`.
struct NotificationSink {
    client: notifications::notification_service_client::NotificationServiceClient<Channel>,
}

impl ThrottleImportStatusSourceSink for NotificationSink {
    fn write(&self, record: ThrottleImportStatusSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_notification(notifications::CreateNotificationRequest {
                    notification: Some(notifications::Notification {
                        subject: record.kind,
                        notification_id: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.secrets.v1.SecretWorkflowService`.
struct SecretWorkflowSink {
    client: secrets::secret_workflow_service_client::SecretWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleImportStatusSourceSink for SecretWorkflowSink {
    fn write(&self, record: ThrottleImportStatusSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::ResumeSecretWorkflowRequest {
                    details: Some(secrets::resume_secret_workflow_request::Details {
                        initiated_by: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.resume_secret_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.ThrottleReportingService`.
struct ThrottleReportingSink {
    client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient<Channel>,
}

impl ThrottleImportStatusSourceSink for ThrottleReportingSink {
    fn write(&self, record: ThrottleImportStatusSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .fetch_throttle_report(ratelimiting::FetchThrottleReportRequest {
                    throttle_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
