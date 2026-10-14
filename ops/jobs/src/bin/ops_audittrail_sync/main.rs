//! Command ops_audittrail_sync syncs audittrail's records between the services that hold them.

mod backfill_crypto_key_id;
mod mirror_crypto_key_rules_code;
mod carry_stock_items_status;
mod copy_key_rotation_id;
mod backfill_throttle_conflict_id;
mod sync_access_grant_eligibility_id;
mod settle_show_rates_id;
mod relay_tickets_id;
mod export_access_policy_name;
mod forward_environment_expiry_code;
mod hand_off_pending_access_grants_id;
mod relay_box_office_sales_token;
mod forward_flag_rollout_version_status;
mod forward_ledger_accounts_timezone;
mod forward_csr_id;
mod replay_broadcasts_token;
mod forward_route_binding_watchers_id;
mod settle_warehouses_uri;
mod backfill_crypto_key_code;
mod replay_anomalies_id;
mod settle_appointment_id;
mod route_gateway_policy_watchers_status;
mod replay_quota_bucket_conflict_status;
mod hand_off_stock_items_status;
mod sync_metric_series_id;
mod backfill_admissions_id;
mod relay_flag_rollout_id;
mod forward_feature_flag_version_name;
mod relay_route_binding_status;
mod route_config_set_workflow_id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = backfill_crypto_key_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill crypto key id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_crypto_key_rules_code::run(channel.clone(), context.clone()).await {
        eprintln!("mirror crypto key rules code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_stock_items_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry stock items status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_key_rotation_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy key rotation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_throttle_conflict_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill throttle conflict id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_access_grant_eligibility_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync access grant eligibility id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_show_rates_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle show rates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_tickets_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay tickets id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_access_policy_name::run(channel.clone(), context.clone()).await {
        eprintln!("export access policy name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_environment_expiry_code::run(channel.clone(), context.clone()).await {
        eprintln!("forward environment expiry code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_pending_access_grants_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off pending access grants id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_box_office_sales_token::run(channel.clone(), context.clone()).await {
        eprintln!("relay box office sales token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_flag_rollout_version_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward flag rollout version status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_ledger_accounts_timezone::run(channel.clone(), context.clone()).await {
        eprintln!("forward ledger accounts timezone failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_csr_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward csr id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_broadcasts_token::run(channel.clone(), context.clone()).await {
        eprintln!("replay broadcasts token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_route_binding_watchers_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward route binding watchers id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_warehouses_uri::run(channel.clone(), context.clone()).await {
        eprintln!("settle warehouses uri failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_crypto_key_code::run(channel.clone(), context.clone()).await {
        eprintln!("backfill crypto key code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_anomalies_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay anomalies id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_appointment_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle appointment id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_gateway_policy_watchers_status::run(channel.clone(), context.clone()).await {
        eprintln!("route gateway policy watchers status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_quota_bucket_conflict_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay quota bucket conflict status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_stock_items_status::run(channel.clone(), context.clone()).await {
        eprintln!("hand off stock items status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_metric_series_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync metric series id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_admissions_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill admissions id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_flag_rollout_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay flag rollout id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_feature_flag_version_name::run(channel.clone(), context.clone()).await {
        eprintln!("forward feature flag version name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_route_binding_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay route binding status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_config_set_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("route config set workflow id failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
