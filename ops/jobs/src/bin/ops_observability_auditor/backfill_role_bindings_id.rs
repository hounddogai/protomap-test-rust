//! `backfill_role_bindings_id` reads `acme.authorization.v1.RoleBindingValidationService/LintRoleBindings`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient::new(channel.clone());
    let found = source
        .lint_role_bindings(authorization::LintRoleBindingsRequest {
            context: Some(context.clone()),
            role_binding_id: "main-backfill-role-bindings-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let role_binding = found.role_binding.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = RoleBindingsIdRecord {
        role_binding_id: acme_kit::text::normalize(&role_binding.role_binding_id),
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
    };
    deliver_role_bindings_id("patient-home", record, channel, &context).await
}

/// What `backfill_role_bindings_id` keeps of each LintRoleBindings response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingsIdRecord {
    pub role_binding_id: String,
    pub message: String,
    pub status: String,
}

/// Where `backfill_role_bindings_id` hands each record.
trait RoleBindingsIdTarget {
    fn send(&self, record: RoleBindingsIdRecord) -> Delivery;
}

/// Sends records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingsIdTarget for PatientHomeSink {
    fn send(&self, record: RoleBindingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patientapp::ClonePatientHomeRequest {
                    request_id: record.status,
                    patient_home: Some(patientapp::PatientHome {
                        subject: record.role_binding_id,
                        patient_home_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_patient_home(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.ApiKeyNotificationService`.
struct ApiKeyNotificationSink {
    client: edgegateway::api_key_notification_service_client::ApiKeyNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingsIdTarget for ApiKeyNotificationSink {
    fn send(&self, record: RoleBindingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::ListApiKeySubscriptionsRequest {
                    filter: Some(edgegateway::list_api_key_subscriptions_request::Filter {
                        region: record.role_binding_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_api_key_subscriptions(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ledger.v1.LedgerAccountValidationService`.
struct LedgerAccountValidationSink {
    client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient<Channel>,
}

impl RoleBindingsIdTarget for LedgerAccountValidationSink {
    fn send(&self, record: RoleBindingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_ledger_account(ledger::NormalizeLedgerAccountRequest {
                    ledger_account_id: record.message,
                    reason: record.role_binding_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.catalog.v1.ProductCalculationService`.
struct ProductCalculationSink {
    client: catalog::product_calculation_service_client::ProductCalculationServiceClient<Channel>,
}

impl RoleBindingsIdTarget for ProductCalculationSink {
    fn send(&self, record: RoleBindingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_product(catalog::SimulateProductRequest {
                    product: Some(catalog::Product {
                        name: record.status,
                        product_id: record.role_binding_id,
                        code: record.message,
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
struct RoleBindingsIdLogOnly;

impl RoleBindingsIdTarget for RoleBindingsIdLogOnly {
    fn send(&self, record: RoleBindingsIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.role_binding_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_role_bindings_id(
    kind: &str,
    record: RoleBindingsIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn RoleBindingsIdTarget> = match kind {
        "patient-home" => Box::new(PatientHomeSink {
            client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "api-key-notification" => Box::new(ApiKeyNotificationSink {
            client: edgegateway::api_key_notification_service_client::ApiKeyNotificationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "ledger-account-validation" => Box::new(LedgerAccountValidationSink {
            client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "product-calculation" => Box::new(ProductCalculationSink {
            client: catalog::product_calculation_service_client::ProductCalculationServiceClient::new(channel),
        }),
        _ => Box::new(RoleBindingsIdLogOnly),
    };
    sink.send(record).await
}
