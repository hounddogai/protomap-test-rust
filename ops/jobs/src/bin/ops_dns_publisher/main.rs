//! Command ops_dns_publisher publishers dns's records between the services that hold them.

mod copy_get_documents_status;
mod export_venues_status;
mod forward_vehicle_status;
mod forward_rate_limit_workflow_id;
mod replay_throttle_workflows_status;
mod mirror_search_index_code;
mod replay_secret_id;
mod settle_permission_check_test_notice_id;
mod replay_yards_name;
mod export_flag_rollout_message;
mod push_sponsors_status;
mod mirror_checkout_sessions_message;
mod forward_throttle_rates_id;
mod copy_experiment_calculation_id;
mod hand_off_config_set_id;
mod hand_off_dispatch_order_id;
mod sync_key_rotation_eligibility_id;
mod carry_sponsors_id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = copy_get_documents_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy get documents status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_venues_status::run(channel.clone(), context.clone()).await {
        eprintln!("export venues status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_vehicle_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward vehicle status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_rate_limit_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward rate limit workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_throttle_workflows_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay throttle workflows status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_search_index_code::run(channel.clone(), context.clone()).await {
        eprintln!("mirror search index code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_secret_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay secret id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_permission_check_test_notice_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle permission check test notice id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_yards_name::run(channel.clone(), context.clone()).await {
        eprintln!("replay yards name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_flag_rollout_message::run(channel.clone(), context.clone()).await {
        eprintln!("export flag rollout message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_sponsors_status::run(channel.clone(), context.clone()).await {
        eprintln!("push sponsors status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_checkout_sessions_message::run(channel.clone(), context.clone()).await {
        eprintln!("mirror checkout sessions message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_throttle_rates_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward throttle rates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_experiment_calculation_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy experiment calculation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_config_set_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off config set id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_dispatch_order_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off dispatch order id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_key_rotation_eligibility_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync key rotation eligibility id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_sponsors_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry sponsors id failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
