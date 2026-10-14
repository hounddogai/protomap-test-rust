use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_server::FlagRolloutCalculationService;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Computes the figures of a flag rollout, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct FlagRolloutCalculationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    simulate_flag_rollout_0_port:
        std::sync::Arc<dyn acme_platform::ports::RpcPort<venues::CloneVenueRequest, venues::CloneVenueResponse>>,
}
impl FlagRolloutCalculationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<venues::CloneVenueRequest, venues::CloneVenueResponse> = {
            let client = VenueServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: venues::CloneVenueRequest| -> acme_platform::ports::RpcFuture<venues::CloneVenueResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_venue(request).await?.into_inner())
})
})
        };
        let simulate_flag_rollout_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            simulate_flag_rollout_0_port,
        }
    }
    /// Run CalculateFlagRollout after validating the tenant and durably recording the command.
    async fn run_calculate_flag_rollout(
        &self,
        req: featureflags::CalculateFlagRolloutRequest,
    ) -> Result<featureflags::CalculateFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::CalculateFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run EstimateFlagRollout after validating the tenant and durably recording the command.
    async fn run_estimate_flag_rollout(
        &self,
        req: featureflags::EstimateFlagRolloutRequest,
    ) -> Result<featureflags::EstimateFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let outgoing_0 = venues::SearchVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::search_venues_request::Filter {
                created_after: req
                    .flag_rollout
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: (req.flag_rollout.clone().unwrap_or_default().metadata.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
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
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::SearchVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::EstimateFlagRolloutResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_rollout.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CompareFlagRolloutOptions after validating the tenant and durably recording the command.
    async fn run_compare_flag_rollout_options(
        &self,
        req: featureflags::CompareFlagRolloutOptionsRequest,
    ) -> Result<featureflags::CompareFlagRolloutOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareFlagRolloutOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareFlagRolloutOptions", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let outgoing_0 = venues::UpdateVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                locator: Some(venues::venue::Locator::Address(common::PostalAddress {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::UpdateVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::CompareFlagRolloutOptionsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.flag_rollout.clone().unwrap_or_default().window.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateFlagRollouts after validating the tenant and durably recording the command.
    async fn run_recalculate_flag_rollouts(
        &self,
        req: featureflags::RecalculateFlagRolloutsRequest,
    ) -> Result<featureflags::RecalculateFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateFlagRollouts", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::RecalculateFlagRolloutsResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.flag_rollout.clone().unwrap_or_default().organizer.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).organizer = contribution;
        }
        Ok(response)
    }
    /// Run SimulateFlagRollout after validating the tenant and durably recording the command.
    async fn run_simulate_flag_rollout(
        &self,
        req: featureflags::SimulateFlagRolloutRequest,
    ) -> Result<featureflags::SimulateFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let outgoing_0 = venues::CloneVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
                    email: req
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let reply_0 = acme_platform::ports::execute(self.simulate_flag_rollout_0_port.as_ref(), outgoing_0).await?;
        let mut response = featureflags::SimulateFlagRolloutResponse::default();
        let contribution: String = (match reply_0.venue.clone().unwrap_or_default().locator.clone() {
            Some(venues::venue::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .recipient
        .clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ExplainFlagRolloutCalculation after validating the tenant and durably recording the command.
    async fn run_explain_flag_rollout_calculation(
        &self,
        req: featureflags::ExplainFlagRolloutCalculationRequest,
    ) -> Result<featureflags::ExplainFlagRolloutCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainFlagRolloutCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainFlagRolloutCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let jobs_0 = req
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = venues::BrowseVenuesRequest {
                        context: req.context.clone(),
                        filter: Some(venues::browse_venues_request::Filter {
                            owner_id: item.participant_id.clone(),
                            region: req
                                .flag_rollout
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .region
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = VenueQueryServiceClient::new(channel);
                        let input: tonic::Request<venues::BrowseVenuesRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.browse_venues(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = featureflags::ExplainFlagRolloutCalculationResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.flag_rollout.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagRolloutRates after validating the tenant and durably recording the command.
    async fn run_get_flag_rollout_rates(
        &self,
        req: featureflags::GetFlagRolloutRatesRequest,
    ) -> Result<featureflags::GetFlagRolloutRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFlagRolloutRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagRolloutRates", "running workflow");
        let mut response = featureflags::GetFlagRolloutRatesResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagRolloutCalculationService for FlagRolloutCalculationController {
    /// Calculates a flag rollout.
    async fn calculate_flag_rollout(
        &self,
        request: tonic::Request<featureflags::CalculateFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::CalculateFlagRolloutResponse>, tonic::Status> {
        self.run_calculate_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a flag rollout.
    async fn estimate_flag_rollout(
        &self,
        request: tonic::Request<featureflags::EstimateFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::EstimateFlagRolloutResponse>, tonic::Status> {
        self.run_estimate_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a flag rollout.
    async fn compare_flag_rollout_options(
        &self,
        request: tonic::Request<featureflags::CompareFlagRolloutOptionsRequest>,
    ) -> Result<tonic::Response<featureflags::CompareFlagRolloutOptionsResponse>, tonic::Status> {
        self.run_compare_flag_rollout_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a flag rollout.
    async fn recalculate_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::RecalculateFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::RecalculateFlagRolloutsResponse>, tonic::Status> {
        self.run_recalculate_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a flag rollout.
    async fn simulate_flag_rollout(
        &self,
        request: tonic::Request<featureflags::SimulateFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::SimulateFlagRolloutResponse>, tonic::Status> {
        self.run_simulate_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a flag rollout.
    async fn explain_flag_rollout_calculation(
        &self,
        request: tonic::Request<featureflags::ExplainFlagRolloutCalculationRequest>,
    ) -> Result<tonic::Response<featureflags::ExplainFlagRolloutCalculationResponse>, tonic::Status> {
        self.run_explain_flag_rollout_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag rollout.
    async fn get_flag_rollout_rates(
        &self,
        request: tonic::Request<featureflags::GetFlagRolloutRatesRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagRolloutRatesResponse>, tonic::Status> {
        self.run_get_flag_rollout_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
