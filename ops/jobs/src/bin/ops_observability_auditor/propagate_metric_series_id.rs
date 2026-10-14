//! `propagate_metric_series_id` reads `acme.metrics.v1.MetricSeriesValidationService/ValidateMetricSeries`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(channel.clone());
    let found = source
        .validate_metric_series(metrics::ValidateMetricSeriesRequest {
            context: Some(context.clone()),
            reason: "primary-propagate-metric-series-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let metric_series = found.metric_series.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = MetricSeriesIdRecord {
        metric_series_id: metric_series.metric_series_id,
        code: metric_series.code,
        message: acme_kit::text::normalize(&result.message),
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn MetricSeriesIdSink>> = BTreeMap::new();
    sinks.insert(
        "access-policy-admin",
        Box::new(AccessPolicyAdminSink {
            client: authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "access-grant-approval",
        Box::new(AccessGrantApprovalSink {
            client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "certificate-calculation",
        Box::new(CertificateCalculationSink {
            client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "user",
        Box::new(UserSink {
            client: identity::user_service_client::UserServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    match sinks.get("access-policy-admin") {
        Some(sink) => sink.forward(record).await,
        None => anyhow::bail!("no sink registered for access-policy-admin"),
    }
}

/// What `propagate_metric_series_id` keeps of each ValidateMetricSeries response.
#[derive(Clone, Debug, Default)]
pub struct MetricSeriesIdRecord {
    pub metric_series_id: String,
    pub code: String,
    pub message: String,
    pub status: String,
}

/// Where `propagate_metric_series_id` hands each record.
trait MetricSeriesIdSink {
    fn forward(&self, record: MetricSeriesIdRecord) -> Delivery;
}

/// Passes records to `acme.authorization.v1.AccessPolicyAdminService`.
struct AccessPolicyAdminSink {
    client: authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient<Channel>,
}

impl MetricSeriesIdSink for AccessPolicyAdminSink {
    fn forward(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .unlock_access_policy(authorization::UnlockAccessPolicyRequest {
                    access_policy_id: record.metric_series_id,
                    details: Some(authorization::unlock_access_policy_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.AccessGrantApprovalService`.
struct AccessGrantApprovalSink {
    client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient<Channel>,
}

impl MetricSeriesIdSink for AccessGrantApprovalSink {
    fn forward(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_pending_access_grants(secrets::ListPendingAccessGrantsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    filter: Some(secrets::list_pending_access_grants_request::Filter {
                        owner_id: record.message,
                        region: record.metric_series_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CertificateCalculationService`.
struct CertificateCalculationSink {
    client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient<Channel>,
}

impl MetricSeriesIdSink for CertificateCalculationSink {
    fn forward(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_certificate_rates(certificates::GetCertificateRatesRequest {
                    certificate_id: record.code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.identity.v1.UserService`.
struct UserSink {
    client: identity::user_service_client::UserServiceClient<Channel>,
    context: common::RequestContext,
}

impl MetricSeriesIdSink for UserSink {
    fn forward(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(identity::UpdateUserRequest {
                    user: Some(identity::User {
                        user_id: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.update_user(request).await?;
            Ok(())
        })
    }
}
