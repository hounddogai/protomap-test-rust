//! Command ops_quotas_sweeper sweepers quotas's records between the services that hold them.

mod carry_quota_bucket_status;
mod backfill_admission_kind;
mod carry_throttle_options_id;
mod copy_flag_evaluation_message;
mod backfill_tours_id;
mod mirror_permission_check_updates_id;
mod replay_api_keys_id;
mod relay_seat_maps_id;
mod backfill_permission_check_id;
mod carry_throttle_conflicts_id;
mod hand_off_sponsor_message;
mod replay_certificate_authority_id;
mod push_quota_bucket_decision_id;
mod backfill_quota_buckets_status;
mod carry_quota_bucket_workflow_code;
mod push_config_change_workflow_source;
mod copy_rate_limits_id;
mod settle_gateway_policy_workflow_name;
mod route_access_grant_code;
mod hand_off_key_ring_code;
mod replay_show_status;
mod forward_media_asset_name;
mod route_anomalies_message;
mod settle_environment_status;
mod mirror_certificate_authority_message;
mod settle_invoice_batch_id;
mod hand_off_certificate_authority_workflow_id;
mod route_crypto_key_lifecycle_id;
mod mirror_secret_id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = carry_quota_bucket_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry quota bucket status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_admission_kind::run(channel.clone(), context.clone()).await {
        eprintln!("backfill admission kind failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_throttle_options_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry throttle options id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_flag_evaluation_message::run(channel.clone(), context.clone()).await {
        eprintln!("copy flag evaluation message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_tours_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill tours id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_permission_check_updates_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror permission check updates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_api_keys_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay api keys id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_seat_maps_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay seat maps id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_permission_check_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill permission check id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_throttle_conflicts_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry throttle conflicts id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_sponsor_message::run(channel.clone(), context.clone()).await {
        eprintln!("hand off sponsor message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_certificate_authority_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay certificate authority id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_quota_bucket_decision_id::run(channel.clone(), context.clone()).await {
        eprintln!("push quota bucket decision id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_quota_buckets_status::run(channel.clone(), context.clone()).await {
        eprintln!("backfill quota buckets status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_quota_bucket_workflow_code::run(channel.clone(), context.clone()).await {
        eprintln!("carry quota bucket workflow code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_config_change_workflow_source::run(channel.clone(), context.clone()).await {
        eprintln!("push config change workflow source failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_rate_limits_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy rate limits id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_gateway_policy_workflow_name::run(channel.clone(), context.clone()).await {
        eprintln!("settle gateway policy workflow name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_access_grant_code::run(channel.clone(), context.clone()).await {
        eprintln!("route access grant code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_key_ring_code::run(channel.clone(), context.clone()).await {
        eprintln!("hand off key ring code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_show_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay show status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_media_asset_name::run(channel.clone(), context.clone()).await {
        eprintln!("forward media asset name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_anomalies_message::run(channel.clone(), context.clone()).await {
        eprintln!("route anomalies message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_environment_status::run(channel.clone(), context.clone()).await {
        eprintln!("settle environment status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_certificate_authority_message::run(channel.clone(), context.clone()).await {
        eprintln!("mirror certificate authority message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_invoice_batch_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle invoice batch id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_certificate_authority_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off certificate authority workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_crypto_key_lifecycle_id::run(channel.clone(), context.clone()).await {
        eprintln!("route crypto key lifecycle id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_secret_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror secret id failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
