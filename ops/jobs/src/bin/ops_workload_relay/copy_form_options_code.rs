//! `copy_form_options_code` reads `acme.forms.v1.FormCalculationService/CompareFormOptions` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::forms::v1 as forms;
use acme_proto::acme::identity::v1 as identity;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = forms::form_calculation_service_client::FormCalculationServiceClient::new(channel.clone());
    let found = source
        .compare_form_options(forms::CompareFormOptionsRequest {
            context: Some(context.clone()),
            form_id: "nightly-copy-form-options-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FormOptionsCodeRecord {
        currency_code: breakdown.currency_code,
        message: result.message,
    };
    let sinks: Vec<(&str, Box<dyn FormOptionsCodeTarget>)> = vec![
        (
            "route-binding-query",
            Box::new(RouteBindingQuerySink {
                client: edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "user-validation",
            Box::new(UserValidationSink {
                client: identity::user_validation_service_client::UserValidationServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "route-binding-query" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `copy_form_options_code` keeps of each CompareFormOptions response.
#[derive(Clone, Debug, Default)]
pub struct FormOptionsCodeRecord {
    pub currency_code: String,
    pub message: String,
}

/// Where `copy_form_options_code` hands each record.
trait FormOptionsCodeTarget {
    fn write(&self, record: FormOptionsCodeRecord) -> Delivery;
}

/// Hands records to `acme.edgegateway.v1.RouteBindingQueryService`.
struct RouteBindingQuerySink {
    client: edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FormOptionsCodeTarget for RouteBindingQuerySink {
    fn write(&self, record: FormOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::LookupRouteBindingRequest {
                    route_binding_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_route_binding(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.identity.v1.UserValidationService`.
struct UserValidationSink {
    client: identity::user_validation_service_client::UserValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl FormOptionsCodeTarget for UserValidationSink {
    fn write(&self, record: FormOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(identity::LintUsersRequest {
                    details: Some(identity::lint_users_request::Details {
                        locale: record.message,
                        ..Default::default()
                    }),
                    reason: record.currency_code,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_users(request).await?;
            Ok(())
        })
    }
}
