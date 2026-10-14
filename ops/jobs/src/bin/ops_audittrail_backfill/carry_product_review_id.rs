//! `carry_product_review_id` reads `acme.reviews.v1.ProductReviewService/GetProductReview` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::payroll::v1 as payroll;
use acme_proto::acme::reviews::v1 as reviews;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = reviews::product_review_service_client::ProductReviewServiceClient::new(channel.clone());
    let reply = source
        .get_product_review(reviews::GetProductReviewRequest {
            context: Some(context.clone()),
            product_review_id: "main-carry-product-review-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let product_review = reply.product_review.unwrap_or_default();
    let record = ProductReviewIdRecord {
        correlation_id: result.correlation_id,
        product_review_id: product_review.product_review_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ProductReviewIdDestination>> = BTreeMap::new();
    sinks.insert(
        "access-grant-validation",
        Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "pay-run",
        Box::new(PayRunSink {
            client: payroll::pay_run_service_client::PayRunServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "role-binding-admin",
        Box::new(RoleBindingAdminSink {
            client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "secret-approval",
        Box::new(SecretApprovalSink {
            client: secrets::secret_approval_service_client::SecretApprovalServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("pay-run") else {
        anyhow::bail!("no sink registered for pay-run");
    };
    sink.apply(record).await
}

/// What `carry_product_review_id` keeps of each GetProductReview response.
#[derive(Clone, Debug, Default)]
pub struct ProductReviewIdRecord {
    pub correlation_id: String,
    pub product_review_id: String,
}

/// Where `carry_product_review_id` hands each record.
trait ProductReviewIdDestination {
    fn apply(&self, record: ProductReviewIdRecord) -> Delivery;
}

/// Sends records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
}

impl ProductReviewIdDestination for AccessGrantValidationSink {
    fn apply(&self, record: ProductReviewIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_access_grant(secrets::ValidateAccessGrantRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.payroll.v1.PayRunService`.
struct PayRunSink {
    client: payroll::pay_run_service_client::PayRunServiceClient<Channel>,
}

impl ProductReviewIdDestination for PayRunSink {
    fn apply(&self, record: ProductReviewIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_pay_runs(payroll::ListPayRunsRequest {
                    filter: Some(payroll::list_pay_runs_request::Filter {
                        region: record.product_review_id,
                        query: record.correlation_id,
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
}

impl ProductReviewIdDestination for RoleBindingAdminSink {
    fn apply(&self, record: ProductReviewIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .restore_role_binding(authorization::RestoreRoleBindingRequest {
                    reason: record.product_review_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.SecretApprovalService`.
struct SecretApprovalSink {
    client: secrets::secret_approval_service_client::SecretApprovalServiceClient<Channel>,
}

impl ProductReviewIdDestination for SecretApprovalSink {
    fn apply(&self, record: ProductReviewIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .escalate_secret(secrets::EscalateSecretRequest {
                    reason: record.product_review_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
