//! `route_config_set_approval_status_name` reads
//! `acme.configuration.v1.ConfigSetApprovalService/GetConfigSetApprovalStatus` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(channel.clone());
    let reply = source
        .get_config_set_approval_status(configuration::GetConfigSetApprovalStatusRequest {
            context: Some(context.clone()),
            config_set_id: "current-route-config-set-approval-status-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_set = reply.config_set.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = ConfigSetApprovalStatusNameRecord {
        name: config_set.name,
        parent_id: config_set.parent_id,
        config_set_id: config_set.config_set_id,
        message: result.message,
    };
    deliver_config_set_approval_status_name("crypto-key-lifecycle", record, channel, &context).await
}

/// What `route_config_set_approval_status_name` keeps of each GetConfigSetApprovalStatus response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetApprovalStatusNameRecord {
    pub name: String,
    pub parent_id: String,
    pub config_set_id: String,
    pub message: String,
}

/// Where `route_config_set_approval_status_name` hands each record.
trait ConfigSetApprovalStatusNameWriter {
    fn apply(&self, record: ConfigSetApprovalStatusNameRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyLifecycleService`.
struct CryptoKeyLifecycleSink {
    client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient<Channel>,
}

impl ConfigSetApprovalStatusNameWriter for CryptoKeyLifecycleSink {
    fn apply(&self, record: ConfigSetApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_crypto_key_lifecycle(keymanagement::GetCryptoKeyLifecycleRequest {
                    crypto_key_id: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.warehousing.v1.WarehouseService`.
struct WarehouseSink {
    client: warehousing::warehouse_service_client::WarehouseServiceClient<Channel>,
}

impl ConfigSetApprovalStatusNameWriter for WarehouseSink {
    fn apply(&self, record: ConfigSetApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_warehouse(warehousing::GetWarehouseRequest {
                    warehouse_id: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetApprovalStatusNameWriter for SponsorQuerySink {
    fn apply(&self, record: ConfigSetApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::SummarizeSponsorsRequest {
                    sponsor_id: record.config_set_id,
                    reason: record.parent_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_sponsors(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ConfigSetApprovalStatusNameLogOnly;

impl ConfigSetApprovalStatusNameWriter for ConfigSetApprovalStatusNameLogOnly {
    fn apply(&self, record: ConfigSetApprovalStatusNameRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.name);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_config_set_approval_status_name(
    kind: &str,
    record: ConfigSetApprovalStatusNameRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn ConfigSetApprovalStatusNameWriter> = match kind {
        "crypto-key-lifecycle" => Box::new(CryptoKeyLifecycleSink {
            client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
        "warehouse" => Box::new(WarehouseSink {
            client: warehousing::warehouse_service_client::WarehouseServiceClient::new(channel.clone()),
        }),
        "sponsor-query" => Box::new(SponsorQuerySink {
            client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(ConfigSetApprovalStatusNameLogOnly),
    };
    sink.apply(record).await
}
