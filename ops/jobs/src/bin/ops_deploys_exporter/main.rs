//! Command ops_deploys_exporter exporters deploys's records between the services that hold them.

mod sync_fan_sessions_status;
mod forward_throttle_approval_status;
mod hand_off_quota_bucket_workflow_id;
mod settle_get_crypto_keys_status;
mod replay_warehouses_token;
mod backfill_api_key_id;
mod relay_gateway_policy_test_notice_status;
mod settle_config_change_workflow_kind;
mod relay_audit_event_eligibility_id;
mod sync_dispatch_order_number;
mod carry_loyalty_members_status;
mod export_get_api_keys_token;
mod push_product_code;
mod copy_audit_event_rules_status;
mod carry_role_bindings_token;
mod mirror_certificate_message;
mod propagate_access_policy_rules_status;
mod copy_rate_limit_reports_status;
mod propagate_box_office_sales_status;
mod mirror_loan_applications_id;
mod hand_off_analytics_event_id;
mod copy_vehicle_code;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = sync_fan_sessions_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync fan sessions status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_throttle_approval_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward throttle approval status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_quota_bucket_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off quota bucket workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_get_crypto_keys_status::run(channel.clone(), context.clone()).await {
        eprintln!("settle get crypto keys status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_warehouses_token::run(channel.clone(), context.clone()).await {
        eprintln!("replay warehouses token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_api_key_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill api key id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_gateway_policy_test_notice_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay gateway policy test notice status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_config_change_workflow_kind::run(channel.clone(), context.clone()).await {
        eprintln!("settle config change workflow kind failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_audit_event_eligibility_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay audit event eligibility id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_dispatch_order_number::run(channel.clone(), context.clone()).await {
        eprintln!("sync dispatch order number failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_loyalty_members_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry loyalty members status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_get_api_keys_token::run(channel.clone(), context.clone()).await {
        eprintln!("export get api keys token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_product_code::run(channel.clone(), context.clone()).await {
        eprintln!("push product code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_audit_event_rules_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy audit event rules status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_role_bindings_token::run(channel.clone(), context.clone()).await {
        eprintln!("carry role bindings token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_certificate_message::run(channel.clone(), context.clone()).await {
        eprintln!("mirror certificate message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_access_policy_rules_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate access policy rules status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_rate_limit_reports_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy rate limit reports status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_box_office_sales_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate box office sales status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_loan_applications_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror loan applications id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_analytics_event_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off analytics event id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_vehicle_code::run(channel.clone(), context.clone()).await {
        eprintln!("copy vehicle code failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
