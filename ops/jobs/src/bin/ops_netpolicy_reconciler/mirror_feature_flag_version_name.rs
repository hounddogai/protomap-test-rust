//! `mirror_feature_flag_version_name` reads
//! `acme.featureflags.v1.FeatureFlagHistoryService/GetFeatureFlagVersion` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ledger::v1 as ledger;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(channel.clone());
    let found = source
        .get_feature_flag_version(featureflags::GetFeatureFlagVersionRequest {
            context: Some(context.clone()),
            feature_flag_id: "primary-mirror-feature-flag-version-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let feature_flag = found.feature_flag.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FeatureFlagVersionNameRecord {
        name: acme_kit::text::normalize(&feature_flag.name),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        feature_flag_id: feature_flag.feature_flag_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn FeatureFlagVersionNameWriter>> = BTreeMap::new();
    sinks.insert("certificate-authority-calculation", Box::new(CertificateAuthorityCalculationSink { client: certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient::new(channel.clone()), context: context.clone() }));
    sinks.insert(
        "box-office-sale-query",
        Box::new(BoxOfficeSaleQuerySink {
            client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "ledger-account",
        Box::new(LedgerAccountSink {
            client: ledger::ledger_account_service_client::LedgerAccountServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("ledger-account") else {
        anyhow::bail!("no sink registered for ledger-account");
    };
    sink.send(record).await
}

/// What `mirror_feature_flag_version_name` keeps of each GetFeatureFlagVersion response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagVersionNameRecord {
    pub name: String,
    pub correlation_id: String,
    pub feature_flag_id: String,
}

/// Where `mirror_feature_flag_version_name` hands each record.
trait FeatureFlagVersionNameWriter {
    fn send(&self, record: FeatureFlagVersionNameRecord) -> Delivery;
}

/// Passes records to `acme.certificates.v1.CertificateAuthorityCalculationService`.
struct CertificateAuthorityCalculationSink {
    client:
        certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient<
            Channel,
        >,
    context: common::RequestContext,
}

impl FeatureFlagVersionNameWriter for CertificateAuthorityCalculationSink {
    fn send(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::RecalculateCertificateAuthoritiesRequest {
                    certificate_authority: Some(certificates::CertificateAuthority {
                        parent_id: record.name,
                        ..Default::default()
                    }),
                    reason: record.feature_flag_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.recalculate_certificate_authorities(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
}

impl FeatureFlagVersionNameWriter for BoxOfficeSaleQuerySink {
    fn send(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_box_office_sales(boxoffice::CountBoxOfficeSalesRequest {
                    filter: Some(boxoffice::count_box_office_sales_request::Filter {
                        query: record.name,
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.feature_flag_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ledger.v1.LedgerAccountService`.
struct LedgerAccountSink {
    client: ledger::ledger_account_service_client::LedgerAccountServiceClient<Channel>,
}

impl FeatureFlagVersionNameWriter for LedgerAccountSink {
    fn send(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_ledger_account(ledger::CreateLedgerAccountRequest {
                    request_id: record.feature_flag_id,
                    ledger_account: Some(ledger::LedgerAccount {
                        ledger_account_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
