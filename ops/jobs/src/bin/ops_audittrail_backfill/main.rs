//! Command ops_audittrail_backfill backfills audittrail's records between the services that hold them.

mod route_get_contracts_status;
mod sync_rate_limit_name;
mod relay_rate_limit_name;
mod backfill_gift_card_status;
mod propagate_get_appointments_status;
mod hand_off_driver_id;
mod mirror_document_id;
mod settle_quota_bucket_options_status;
mod carry_fulfillments_id;
mod forward_seat_map_message;
mod propagate_secret_version_workflow_status;
mod copy_secret_workflow_id;
mod push_seat_map_message;
mod carry_product_review_id;
mod route_config_sets_id;
mod replay_crypto_keys_status;
mod mirror_flag_rollout_decision_id;
mod replay_flag_rollouts_message;
mod mirror_access_grant_workflow_id;
mod hand_off_secret_id;
mod mirror_flag_rollout_id;
mod carry_role_binding_export_id;
mod route_rate_limit_calculation_id;
mod route_quota_bucket_workflows_token;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = route_get_contracts_status::run(channel.clone(), context.clone()).await {
        eprintln!("route get contracts status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_rate_limit_name::run(channel.clone(), context.clone()).await {
        eprintln!("sync rate limit name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_rate_limit_name::run(channel.clone(), context.clone()).await {
        eprintln!("relay rate limit name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_gift_card_status::run(channel.clone(), context.clone()).await {
        eprintln!("backfill gift card status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_get_appointments_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate get appointments status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_driver_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off driver id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_document_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror document id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_quota_bucket_options_status::run(channel.clone(), context.clone()).await {
        eprintln!("settle quota bucket options status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_fulfillments_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry fulfillments id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_seat_map_message::run(channel.clone(), context.clone()).await {
        eprintln!("forward seat map message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_secret_version_workflow_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate secret version workflow status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_secret_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy secret workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_seat_map_message::run(channel.clone(), context.clone()).await {
        eprintln!("push seat map message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_product_review_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry product review id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_config_sets_id::run(channel.clone(), context.clone()).await {
        eprintln!("route config sets id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_crypto_keys_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay crypto keys status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_flag_rollout_decision_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror flag rollout decision id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_flag_rollouts_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay flag rollouts message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_access_grant_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror access grant workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_secret_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off secret id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_flag_rollout_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror flag rollout id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_role_binding_export_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry role binding export id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_rate_limit_calculation_id::run(channel.clone(), context.clone()).await {
        eprintln!("route rate limit calculation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_quota_bucket_workflows_token::run(channel.clone(), context.clone()).await {
        eprintln!("route quota bucket workflows token failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
