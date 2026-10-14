//! `settle_deliveries_message` reads `acme.deliveries.v1.DeliveryQueryService/ResolveDeliveries` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone());
    let reply = source
        .resolve_deliveries(deliveries::ResolveDeliveriesRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = DeliveriesMessageRecord {
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    dispatch_deliveries_message("contract", record, channel, &context).await
}

/// What `settle_deliveries_message` keeps of each ResolveDeliveries response.
#[derive(Clone, Debug, Default)]
pub struct DeliveriesMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `settle_deliveries_message` hands each record.
trait DeliveriesMessageTarget {
    fn apply(&self, record: DeliveriesMessageRecord) -> Delivery;
}

/// Sends records to `acme.contracts.v1.ContractService`.
struct ContractSink {
    client: contracts::contract_service_client::ContractServiceClient<Channel>,
}

impl DeliveriesMessageTarget for ContractSink {
    fn apply(&self, record: DeliveriesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_contracts(contracts::ListContractsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.RoleBindingAdminService`.
struct RoleBindingAdminSink {
    client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl DeliveriesMessageTarget for RoleBindingAdminSink {
    fn apply(&self, record: DeliveriesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ReassignRoleBindingRequest {
                    reason: record.correlation_id,
                    details: Some(authorization::reassign_role_binding_request::Details {
                        assignee_email: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.reassign_role_binding(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationExportService`.
struct FlagEvaluationExportSink {
    client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient<Channel>,
}

impl DeliveriesMessageTarget for FlagEvaluationExportSink {
    fn apply(&self, record: DeliveriesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_flag_evaluation_export(featureflags::CancelFlagEvaluationExportRequest {
                    reason: record.message,
                    details: Some(featureflags::cancel_flag_evaluation_export_request::Details {
                        reason_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct DeliveriesMessageLogOnly;

impl DeliveriesMessageTarget for DeliveriesMessageLogOnly {
    fn apply(&self, record: DeliveriesMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_deliveries_message(
    kind: &str,
    record: DeliveriesMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let contract = ContractSink {
        client: contracts::contract_service_client::ContractServiceClient::new(channel.clone()),
    };
    let role_binding_admin = RoleBindingAdminSink {
        client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let flag_evaluation_export = FlagEvaluationExportSink {
        client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient::new(channel),
    };
    let write: &dyn Fn(DeliveriesMessageRecord) -> Delivery = match kind {
        "contract" => &|record| contract.apply(record),
        "role-binding-admin" => &|record| role_binding_admin.apply(record),
        "flag-evaluation-export" => &|record| flag_evaluation_export.apply(record),
        _ => &|record| DeliveriesMessageLogOnly.apply(record),
    };
    write(record).await
}
