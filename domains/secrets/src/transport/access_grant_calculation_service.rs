use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_client::AccessPolicyQueryServiceClient;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_calculation_service_client::PatientCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_server::AccessGrantCalculationService;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Computes the figures of an access grant, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct AccessGrantCalculationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    calculate_access_grant_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<keymanagement::FindCryptoKeysRequest, keymanagement::FindCryptoKeysResponse>,
    >,
    calculate_access_grant_2_port: acme_platform::ports::Callback<
        patients::ExplainPatientCalculationRequest,
        patients::ExplainPatientCalculationResponse,
    >,
    compare_access_grant_options_1_port:
        acme_platform::ports::Callback<featureflags::LintFeatureFlagsRequest, featureflags::LintFeatureFlagsResponse>,
    quote_access_grant_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<certificates::SimulateCsrRequest, certificates::SimulateCsrResponse>,
    >,
    simulate_access_grant_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<audit::FindAuditEventsRequest, audit::FindAuditEventsResponse>,
    >,
    simulate_access_grant_1_port: acme_platform::ports::Callback<
        authorization::BrowseAccessPoliciesRequest,
        authorization::BrowseAccessPoliciesResponse,
    >,
    simulate_access_grant_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::ExplainQuotaBucketDecisionRequest,
            ratelimiting::ExplainQuotaBucketDecisionResponse,
        >,
    >,
    simulate_access_grant_3_port:
        acme_platform::ports::Callback<audit::ResolveAuditEventsRequest, audit::ResolveAuditEventsResponse>,
}
impl AccessGrantCalculationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::FindCryptoKeysRequest,
            keymanagement::FindCryptoKeysResponse,
        > = {
            let client = CryptoKeyQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::FindCryptoKeysRequest| -> acme_platform::ports::RpcFuture<keymanagement::FindCryptoKeysResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_crypto_keys(request).await?.into_inner())
})
})
        };
        let calculate_access_grant_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            patients::ExplainPatientCalculationRequest,
            patients::ExplainPatientCalculationResponse,
        > = {
            let client = PatientCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: patients::ExplainPatientCalculationRequest| -> acme_platform::ports::RpcFuture<patients::ExplainPatientCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_patient_calculation(request).await?.into_inner())
})
})
        };
        let calculate_access_grant_2_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::SimulateCsrRequest,
            certificates::SimulateCsrResponse,
        > = {
            let client = CsrCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::SimulateCsrRequest| -> acme_platform::ports::RpcFuture<certificates::SimulateCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_csr(request).await?.into_inner())
})
})
        };
        let quote_access_grant_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::LintFeatureFlagsRequest,
            featureflags::LintFeatureFlagsResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::LintFeatureFlagsRequest| -> acme_platform::ports::RpcFuture<featureflags::LintFeatureFlagsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_feature_flags(request).await?.into_inner())
})
})
        };
        let compare_access_grant_options_1_port = callback;
        let callback: acme_platform::ports::Callback<audit::FindAuditEventsRequest, audit::FindAuditEventsResponse> = {
            let client = AuditEventQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::FindAuditEventsRequest| -> acme_platform::ports::RpcFuture<audit::FindAuditEventsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_audit_events(request).await?.into_inner())
})
})
        };
        let simulate_access_grant_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            authorization::BrowseAccessPoliciesRequest,
            authorization::BrowseAccessPoliciesResponse,
        > = {
            let client = AccessPolicyQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::BrowseAccessPoliciesRequest| -> acme_platform::ports::RpcFuture<authorization::BrowseAccessPoliciesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_access_policies(request).await?.into_inner())
})
})
        };
        let simulate_access_grant_1_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ExplainQuotaBucketDecisionRequest,
            ratelimiting::ExplainQuotaBucketDecisionResponse,
        > = {
            let client = QuotaBucketValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ExplainQuotaBucketDecisionRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ExplainQuotaBucketDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_quota_bucket_decision(request).await?.into_inner())
})
})
        };
        let simulate_access_grant_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            audit::ResolveAuditEventsRequest,
            audit::ResolveAuditEventsResponse,
        > = {
            let client = AuditEventQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::ResolveAuditEventsRequest| -> acme_platform::ports::RpcFuture<audit::ResolveAuditEventsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_audit_events(request).await?.into_inner())
})
})
        };
        let simulate_access_grant_3_port = callback;
        Self {
            channel,
            repository,
            calculate_access_grant_0_port,
            calculate_access_grant_2_port,
            compare_access_grant_options_1_port,
            quote_access_grant_1_port,
            simulate_access_grant_0_port,
            simulate_access_grant_1_port,
            simulate_access_grant_2_port,
            simulate_access_grant_3_port,
        }
    }
    /// Run CalculateAccessGrant after validating the tenant and durably recording the command.
    async fn run_calculate_access_grant(
        &self,
        req: secrets::CalculateAccessGrantRequest,
    ) -> Result<secrets::CalculateAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::FindCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::find_crypto_keys_request::Filter {
                created_after: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.calculate_access_grant_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = boxoffice::ExportBoxOfficeSalesRequest {
            context: req.context.clone(),
            filter: Some(boxoffice::export_box_office_sales_request::Filter {
                created_after: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.annotations.clone(),
                owner_id: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = BoxOfficeSaleQueryServiceClient::new(channel);
            let input: tonic::Request<boxoffice::ExportBoxOfficeSalesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_box_office_sales(input).await?;
            let mut frames = response.into_inner();
            let mut last = boxoffice::ExportBoxOfficeSalesResponse::default();
            // The caller asks for a bounded snapshot, then drops its subscription.
            for _ in 0..8 {
                match frames.message().await? {
                    Some(frame) => last = frame,
                    None => break,
                }
            }
            Ok::<_, tonic::Status>(last)
        }
        .await?;
        let outgoing_2 = patients::ExplainPatientCalculationRequest {
            context: req.context.clone(),
            patient: Some(patients::Patient {
                audit: Some(common::AuditInfo {
                    etag: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                primary_address: Some(common::PostalAddress {
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                relationship_manager: Some(identity::UserRef {
                    display_name: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = (self.calculate_access_grant_2_port)(outgoing_2).await?;
        let mut response = secrets::CalculateAccessGrantResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run EstimateAccessGrant after validating the tenant and durably recording the command.
    async fn run_estimate_access_grant(
        &self,
        req: secrets::EstimateAccessGrantRequest,
    ) -> Result<secrets::EstimateAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = venues::NormalizeVenueRequest {
            context: req.context.clone(),
            details: Some(venues::normalize_venue_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            venue: Some(venues::Venue {
                locator: Some(venues::venue::Locator::Address(common::PostalAddress {
                    recipient: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<venues::NormalizeVenueRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::NormalizeVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_1 = EstimateAccessGrantDispatch1::from(req.clone());
        let outgoing_1: configuration::CheckEnvironmentEligibilityRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::CheckEnvironmentEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_environment_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = audit::SuggestAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::suggest_audit_events_request::Filter {
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.audit.v1.SuggestAuditEventsRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::SuggestAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::EstimateAccessGrantResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run QuoteAccessGrant after validating the tenant and durably recording the command.
    async fn run_quote_access_grant(
        &self,
        req: secrets::QuoteAccessGrantRequest,
    ) -> Result<secrets::QuoteAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ValidateFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                action: Some(featureflags::feature_flag::Action::Discount(
                    featureflags::FeatureFlagDiscountAction {
                        percent: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                            Some(secrets::access_grant::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .percent
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ValidateFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::SimulateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                audit: Some(common::AuditInfo {
                    etag: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.quote_access_grant_1_port.as_ref(), outgoing_1).await?;
        let mut response = secrets::QuoteAccessGrantResponse::default();
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: i32 = reply_1.csr.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_grant.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run CompareAccessGrantOptions after validating the tenant and durably recording the command.
    async fn run_compare_access_grant_options(
        &self,
        req: secrets::CompareAccessGrantOptionsRequest,
    ) -> Result<secrets::CompareAccessGrantOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareAccessGrantOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareAccessGrantOptions", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::NormalizeEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::normalize_environment_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    lines: (req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone())
                    .into_iter()
                    .filter_map(|value| {
                        let value = value.trim();
                        if value.is_empty() { None } else { Some(value.to_owned()) }
                    })
                    .collect(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    display_name: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::NormalizeEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::LintFeatureFlagsRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_at: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.compare_access_grant_options_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = secrets::CompareAccessGrantOptionsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: Option<identity::UserRef> = reply_1.feature_flag.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.access_grant.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: String = reply_1.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: i64 = req.access_grant.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.access_grant.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run SimulateAccessGrant after validating the tenant and durably recording the command.
    async fn run_simulate_access_grant(
        &self,
        req: secrets::SimulateAccessGrantRequest,
    ) -> Result<secrets::SimulateAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = audit::FindAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::find_audit_events_request::Filter {
                created_after: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.simulate_access_grant_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = authorization::BrowseAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::browse_access_policies_request::Filter {
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = (self.simulate_access_grant_1_port)(outgoing_1).await?;
        let outgoing_2 = ratelimiting::ExplainQuotaBucketDecisionRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                limits: vec![ratelimiting::QuotaBucketLimit {
                    amount: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Discount(value)) => value,
                        _ => Default::default(),
                    })
                    .amount
                    .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            self.simulate_access_grant_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let outgoing_3 = audit::ResolveAuditEventsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.simulate_access_grant_3_port.clone());
        let reply_3 = acme_platform::ports::execute(&port, outgoing_3).await?;
        let mut response = secrets::SimulateAccessGrantResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.access_grant.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.access_grant.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run ExplainAccessGrantCalculation after validating the tenant and durably recording the command.
    async fn run_explain_access_grant_calculation(
        &self,
        req: secrets::ExplainAccessGrantCalculationRequest,
    ) -> Result<secrets::ExplainAccessGrantCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainAccessGrantCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainAccessGrantCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::CountAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::count_access_policies_request::Filter {
                created_after: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: (req.annotations.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessPolicyQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::CountAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ExplainAccessGrantCalculationResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: i32 = req.access_grant.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run GetAccessGrantRates after validating the tenant and durably recording the command.
    async fn run_get_access_grant_rates(
        &self,
        req: secrets::GetAccessGrantRatesRequest,
    ) -> Result<secrets::GetAccessGrantRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetAccessGrantRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetAccessGrantRates", "running workflow");
        let outgoing_0 = keymanagement::SuggestCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::suggest_crypto_keys_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.access_grant_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SuggestCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::CalculateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CalculateEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::GetAccessGrantRatesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .environment
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by EstimateAccessGrant for CheckEnvironmentEligibility.
struct EstimateAccessGrantDispatch1 {
    context: Option<common::RequestContext>,
    environment_address_region: String,
    environment_owner_user_id: String,
}
impl From<secrets::EstimateAccessGrantRequest> for EstimateAccessGrantDispatch1 {
    fn from(req: secrets::EstimateAccessGrantRequest) -> Self {
        Self {
            context: req.context.clone(),
            environment_address_region: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .region
                .clone(),
            environment_owner_user_id: req
                .access_grant
                .clone()
                .unwrap_or_default()
                .owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
        }
    }
}
impl From<EstimateAccessGrantDispatch1> for configuration::CheckEnvironmentEligibilityRequest {
    fn from(value: EstimateAccessGrantDispatch1) -> Self {
        configuration::CheckEnvironmentEligibilityRequest {
            context: value.context,
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: value.environment_address_region,
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    user_id: value.environment_owner_user_id,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl AccessGrantCalculationService for AccessGrantCalculationApi {
    /// Calculates an access grant.
    async fn calculate_access_grant(
        &self,
        request: tonic::Request<secrets::CalculateAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::CalculateAccessGrantResponse>, tonic::Status> {
        self.run_calculate_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates an access grant.
    async fn estimate_access_grant(
        &self,
        request: tonic::Request<secrets::EstimateAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::EstimateAccessGrantResponse>, tonic::Status> {
        self.run_estimate_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes an access grant.
    async fn quote_access_grant(
        &self,
        request: tonic::Request<secrets::QuoteAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::QuoteAccessGrantResponse>, tonic::Status> {
        self.run_quote_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares an access grant.
    async fn compare_access_grant_options(
        &self,
        request: tonic::Request<secrets::CompareAccessGrantOptionsRequest>,
    ) -> Result<tonic::Response<secrets::CompareAccessGrantOptionsResponse>, tonic::Status> {
        self.run_compare_access_grant_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates an access grant.
    async fn simulate_access_grant(
        &self,
        request: tonic::Request<secrets::SimulateAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::SimulateAccessGrantResponse>, tonic::Status> {
        self.run_simulate_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains an access grant.
    async fn explain_access_grant_calculation(
        &self,
        request: tonic::Request<secrets::ExplainAccessGrantCalculationRequest>,
    ) -> Result<tonic::Response<secrets::ExplainAccessGrantCalculationResponse>, tonic::Status> {
        self.run_explain_access_grant_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access grant.
    async fn get_access_grant_rates(
        &self,
        request: tonic::Request<secrets::GetAccessGrantRatesRequest>,
    ) -> Result<tonic::Response<secrets::GetAccessGrantRatesResponse>, tonic::Status> {
        self.run_get_access_grant_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
