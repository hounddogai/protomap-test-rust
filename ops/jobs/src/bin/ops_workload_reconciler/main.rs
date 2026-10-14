//! Command ops_workload_reconciler reconcilers workload's records between the services that hold them.

mod mirror_stock_items_status;
mod sync_vehicles_status;
mod propagate_appointments_token;
mod forward_access_policies_token;
mod replay_environment_code;
mod push_secret_approval_status_id;
mod propagate_crypto_key_status;
mod propagate_throttle_expiry_id;
mod carry_yard_id;
mod export_documents_token;
mod sync_permission_check_rules_status;
mod replay_appointments_status;
mod replay_video_visits_token;
mod settle_environment_report_name;
mod replay_config_set_workflow_id;
mod carry_ledger_accounts_token;
mod propagate_yards_id;
mod propagate_key_rotation_title;
mod propagate_environment_reports_message;
mod settle_labeling_task_id;
mod hand_off_gateway_policies_status;
mod forward_certificate_authorities_id;
mod push_config_change_workflow_message;
mod sync_sponsor_code;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = mirror_stock_items_status::run(channel.clone(), context.clone()).await {
        eprintln!("mirror stock items status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_vehicles_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync vehicles status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_appointments_token::run(channel.clone(), context.clone()).await {
        eprintln!("propagate appointments token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_access_policies_token::run(channel.clone(), context.clone()).await {
        eprintln!("forward access policies token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_environment_code::run(channel.clone(), context.clone()).await {
        eprintln!("replay environment code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_secret_approval_status_id::run(channel.clone(), context.clone()).await {
        eprintln!("push secret approval status id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_crypto_key_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate crypto key status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_throttle_expiry_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate throttle expiry id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_yard_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry yard id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_documents_token::run(channel.clone(), context.clone()).await {
        eprintln!("export documents token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_permission_check_rules_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync permission check rules status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_appointments_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay appointments status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_video_visits_token::run(channel.clone(), context.clone()).await {
        eprintln!("replay video visits token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_environment_report_name::run(channel.clone(), context.clone()).await {
        eprintln!("settle environment report name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_config_set_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay config set workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_ledger_accounts_token::run(channel.clone(), context.clone()).await {
        eprintln!("carry ledger accounts token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_yards_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate yards id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_key_rotation_title::run(channel.clone(), context.clone()).await {
        eprintln!("propagate key rotation title failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_environment_reports_message::run(channel.clone(), context.clone()).await {
        eprintln!("propagate environment reports message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_labeling_task_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle labeling task id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_gateway_policies_status::run(channel.clone(), context.clone()).await {
        eprintln!("hand off gateway policies status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_certificate_authorities_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward certificate authorities id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_config_change_workflow_message::run(channel.clone(), context.clone()).await {
        eprintln!("push config change workflow message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_sponsor_code::run(channel.clone(), context.clone()).await {
        eprintln!("sync sponsor code failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
