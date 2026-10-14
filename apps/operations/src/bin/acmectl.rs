//! Administrative reads and raw unary RPC invocation for platform operators.

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_service_client::CertificateServiceClient;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::route_binding_gateway_service_client::RouteBindingGatewayServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_service_client::FeatureFlagServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;

use acme_operations::RawCodec;
use anyhow::Context;
use prost::Message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut args = std::env::args().skip(1);
    match args.next().context("expected a command or invoke")?.as_str() {
        "get-access-policy" => {
            let mut client = AccessPolicyServiceClient::new(channel);
            let request = authorization::GetAccessPolicyRequest {
                context: Some(context.clone()),
                access_policy_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_access_policy(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-certificate" => {
            let mut client = CertificateServiceClient::new(channel);
            let request = certificates::GetCertificateRequest {
                context: Some(context.clone()),
                certificate_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_certificate(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-config-set" => {
            let mut client = ConfigSetServiceClient::new(channel);
            let request = configuration::GetConfigSetRequest {
                context: Some(context.clone()),
                config_set_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_config_set(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-route-binding-summary" => {
            let mut client = RouteBindingGatewayServiceClient::new(channel);
            let request = edgegateway::GetRouteBindingSummaryRequest {
                context: Some(context.clone()),
                route_binding_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_route_binding_summary(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-feature-flag" => {
            let mut client = FeatureFlagServiceClient::new(channel);
            let request = featureflags::GetFeatureFlagRequest {
                context: Some(context.clone()),
                feature_flag_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_feature_flag(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-crypto-key" => {
            let mut client = CryptoKeyServiceClient::new(channel);
            let request = keymanagement::GetCryptoKeyRequest {
                context: Some(context.clone()),
                crypto_key_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_crypto_key(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-rate-limit" => {
            let mut client = RateLimitServiceClient::new(channel);
            let request = ratelimiting::GetRateLimitRequest {
                context: Some(context.clone()),
                rate_limit_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_rate_limit(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "get-secret" => {
            let mut client = SecretServiceClient::new(channel);
            let request = secrets::GetSecretRequest {
                context: Some(context.clone()),
                secret_id: args.next().context("expected a record id")?,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
            let response = client.get_secret(request).await?.into_inner();
            let output = args
                .next()
                .context("expected an output file for the protobuf response")?;
            std::fs::write(output, response.encode_to_vec())?;
        }
        "invoke" => {
            let path: http::uri::PathAndQuery = args.next().context("expected /package.Service/Method")?.parse()?;
            let payload = std::fs::read(args.next().context("expected a protobuf request file")?)?;
            let output = args.next().context("expected a response output file")?;
            let request = acme_platform::auth::service_request(tonic::Request::new(payload), &context)?;
            let mut grpc = tonic::client::Grpc::new(channel);
            grpc.ready().await?;
            let response = grpc.unary(request, path, RawCodec).await?;
            std::fs::write(output, response.into_inner())?;
        }
        _ => anyhow::bail!("unknown command"),
    }
    Ok(())
}
