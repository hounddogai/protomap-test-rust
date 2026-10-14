//! Command ops_quotas_replicator replicators quotas's records between the services that hold them.

mod settle_quota_buckets_message;
mod replay_access_grant_approval_status_name;
mod sync_box_office_sales_id;
mod forward_environment_lifecycle_id;
mod carry_config_set_id;
mod route_route_binding_name;
mod replay_gateway_policy_updates_id;
mod copy_certificates_status;
mod mirror_crypto_key_message;
mod settle_appointments_token;
mod backfill_secret_version_workflow_kind;
mod backfill_flag_rollouts_code;
mod forward_feature_flag_conflicts_status;
mod export_product_reviews_token;
mod export_ledger_accounts_status;
mod sync_onboarding_plan_id;
mod forward_box_office_sale_status;
mod export_venues_message;
mod carry_gateway_policy_report_message;
mod export_crypto_key_status;
mod route_loyalty_members_status;
mod settle_care_plans_id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = settle_quota_buckets_message::run(channel.clone(), context.clone()).await {
        eprintln!("settle quota buckets message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_access_grant_approval_status_name::run(channel.clone(), context.clone()).await {
        eprintln!("replay access grant approval status name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_box_office_sales_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync box office sales id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_environment_lifecycle_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward environment lifecycle id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_config_set_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry config set id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_route_binding_name::run(channel.clone(), context.clone()).await {
        eprintln!("route route binding name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_gateway_policy_updates_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay gateway policy updates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_certificates_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy certificates status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_crypto_key_message::run(channel.clone(), context.clone()).await {
        eprintln!("mirror crypto key message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_appointments_token::run(channel.clone(), context.clone()).await {
        eprintln!("settle appointments token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_secret_version_workflow_kind::run(channel.clone(), context.clone()).await {
        eprintln!("backfill secret version workflow kind failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_flag_rollouts_code::run(channel.clone(), context.clone()).await {
        eprintln!("backfill flag rollouts code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_feature_flag_conflicts_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward feature flag conflicts status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_product_reviews_token::run(channel.clone(), context.clone()).await {
        eprintln!("export product reviews token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_ledger_accounts_status::run(channel.clone(), context.clone()).await {
        eprintln!("export ledger accounts status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_onboarding_plan_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync onboarding plan id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_box_office_sale_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward box office sale status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_venues_message::run(channel.clone(), context.clone()).await {
        eprintln!("export venues message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_gateway_policy_report_message::run(channel.clone(), context.clone()).await {
        eprintln!("carry gateway policy report message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_crypto_key_status::run(channel.clone(), context.clone()).await {
        eprintln!("export crypto key status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_loyalty_members_status::run(channel.clone(), context.clone()).await {
        eprintln!("route loyalty members status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_care_plans_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle care plans id failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
