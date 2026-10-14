use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_query_service_client::AdmissionQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::access_grant_validation_service_client::AccessGrantValidationServiceClient;
use acme_proto::acme::secrets::v1::secret_query_service_server::SecretQueryService;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_query_service_client::TourQueryServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Answers read-only questions about secrets, for screens and reports.
#[derive(Clone)]
pub struct SecretQueryApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    browse_secrets_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<admissions::CountAdmissionsRequest, admissions::CountAdmissionsResponse>,
    >,
    count_secrets_0_port: acme_platform::ports::Callback<
        authorization::ListAccessPoliciesRequest,
        authorization::ListAccessPoliciesResponse,
    >,
}
impl SecretQueryApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            admissions::CountAdmissionsRequest,
            admissions::CountAdmissionsResponse,
        > = {
            let client = AdmissionQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: admissions::CountAdmissionsRequest| -> acme_platform::ports::RpcFuture<admissions::CountAdmissionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_admissions(request).await?.into_inner())
})
})
        };
        let browse_secrets_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            authorization::ListAccessPoliciesRequest,
            authorization::ListAccessPoliciesResponse,
        > = {
            let client = AccessPolicyServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ListAccessPoliciesRequest| -> acme_platform::ports::RpcFuture<authorization::ListAccessPoliciesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_access_policies(request).await?.into_inner())
})
})
        };
        let count_secrets_0_port = callback;
        Self {
            channel,
            repository,
            browse_secrets_0_port,
            count_secrets_0_port,
        }
    }
    /// Run LookupSecret after validating the tenant and durably recording the command.
    async fn run_lookup_secret(
        &self,
        req: secrets::LookupSecretRequest,
    ) -> Result<secrets::LookupSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupSecret", "running workflow");
        let outgoing_0 = secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
                    user_id: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                parameters: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::ValidateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
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
                action: Some(secrets::access_grant::Action::Block(secrets::AccessGrantBlockAction {
                    reason: (match reply_0.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Block(value)) => value,
                        _ => Default::default(),
                    })
                    .reason
                    .clone(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    display_name: reply_0
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = AccessGrantValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::ValidateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = secrets::LookupSecretResponse::default();
        let contribution: String = reply_0.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseSecrets after validating the tenant and durably recording the command.
    async fn run_browse_secrets(
        &self,
        req: secrets::BrowseSecretsRequest,
    ) -> Result<secrets::BrowseSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = admissions::CountAdmissionsRequest {
            context: req.context.clone(),
            filter: Some(admissions::count_admissions_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_size: req.page.clone().unwrap_or_default().page_size.clone(),
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.browse_secrets_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = secrets::BrowseSecretsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run FindSecrets after validating the tenant and durably recording the command.
    async fn run_find_secrets(
        &self,
        req: secrets::FindSecretsRequest,
    ) -> Result<secrets::FindSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::CountConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_changes_request::Filter {
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.CountConfigChangesRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::CountConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::FindSecretsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run CountSecrets after validating the tenant and durably recording the command.
    async fn run_count_secrets(
        &self,
        req: secrets::CountSecretsRequest,
    ) -> Result<secrets::CountSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = authorization::ListAccessPoliciesRequest {
            context: req.context.clone(),
            page: Some(common::PageRequest {
                page_size: req.page.clone().unwrap_or_default().page_size.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.count_secrets_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = secrets::CountSecretsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ExportSecrets after validating the tenant and durably recording the command.
    async fn run_export_secrets(
        &self,
        req: secrets::ExportSecretsRequest,
    ) -> Result<secrets::ExportSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = tours::CountToursRequest {
            context: req.context.clone(),
            filter: Some(tours::count_tours_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = venues::SearchVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::search_venues_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = TourQueryServiceClient::new(channel_0);
                let input: tonic::Request<tours::CountToursRequest> = tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_tours(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = VenueServiceClient::new(channel_1);
                let input: tonic::Request<venues::SearchVenuesRequest> = tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.search_venues(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = secrets::ExportSecretsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.filter.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run SummarizeSecrets after validating the tenant and durably recording the command.
    async fn run_summarize_secrets(
        &self,
        req: secrets::SummarizeSecretsRequest,
    ) -> Result<secrets::SummarizeSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::CompareAccessGrantOptionsRequest {
            access_grant: Some(secrets::AccessGrant {
                action: Some(secrets::access_grant::Action::Block(secrets::AccessGrantBlockAction {
                    reason: req.reason.clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::CompareAccessGrantOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_access_grant_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::SummarizeSecretsResponse::default();
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
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run PeekSecret after validating the tenant and durably recording the command.
    async fn run_peek_secret(
        &self,
        req: secrets::PeekSecretRequest,
    ) -> Result<secrets::PeekSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekSecret", "running workflow");
        let mut response = secrets::PeekSecretResponse::default();
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl SecretQueryService for SecretQueryApi {
    /// Returns a secret.
    async fn lookup_secret(
        &self,
        request: tonic::Request<secrets::LookupSecretRequest>,
    ) -> Result<tonic::Response<secrets::LookupSecretResponse>, tonic::Status> {
        self.run_lookup_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secrets a page at a time.
    async fn browse_secrets(
        &self,
        request: tonic::Request<secrets::BrowseSecretsRequest>,
    ) -> Result<tonic::Response<secrets::BrowseSecretsResponse>, tonic::Status> {
        self.run_browse_secrets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secrets a page at a time.
    async fn find_secrets(
        &self,
        request: tonic::Request<secrets::FindSecretsRequest>,
    ) -> Result<tonic::Response<secrets::FindSecretsResponse>, tonic::Status> {
        self.run_find_secrets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts secrets by status.
    async fn count_secrets(
        &self,
        request: tonic::Request<secrets::CountSecretsRequest>,
    ) -> Result<tonic::Response<secrets::CountSecretsResponse>, tonic::Status> {
        self.run_count_secrets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportSecretsStream =
        tokio_stream::wrappers::ReceiverStream<Result<secrets::ExportSecretsResponse, tonic::Status>>;
    /// Streams every matching secret.
    async fn export_secrets(
        &self,
        request: tonic::Request<secrets::ExportSecretsRequest>,
    ) -> Result<tonic::Response<Self::ExportSecretsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_export_secrets(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a secret.
    async fn summarize_secrets(
        &self,
        request: tonic::Request<secrets::SummarizeSecretsRequest>,
    ) -> Result<tonic::Response<secrets::SummarizeSecretsResponse>, tonic::Status> {
        self.run_summarize_secrets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a secret.
    async fn peek_secret(
        &self,
        request: tonic::Request<secrets::PeekSecretRequest>,
    ) -> Result<tonic::Response<secrets::PeekSecretResponse>, tonic::Status> {
        self.run_peek_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
