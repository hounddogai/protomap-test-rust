use super::*;

use acme_kit::text;

/// How screens and exports show a pending ttl policy budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyBudgetView {
    pub code: String,
    pub label: String,
    pub score: String,
    pub window_days: String,
    pub ratio_text: String,
    pub checksum_text: String,
    pub rotation_days: String,
    pub key_id: String,
    pub channel_text: String,
    pub cipher_text: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_pending_ttl_policy_budget_view(item: &PendingTtlPolicyBudget) -> PendingTtlPolicyBudgetView {
    PendingTtlPolicyBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        window_days: item.window_days.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        checksum_text: text::truncate(&item.checksum, 40),
        rotation_days: item.rotation_days.to_string(),
        key_id: item.key_id.clone(),
        channel_text: text::truncate(&item.channel, 80),
        cipher_text: text::truncate(&item.cipher, 64),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_ttl_policy_budget_views(items: &[PendingTtlPolicyBudget]) -> Vec<PendingTtlPolicyBudgetView> {
    let mut views: Vec<PendingTtlPolicyBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_ttl_policy_budget_view(item));
    }
    views
}

pub fn sum_pending_ttl_policy_budget_view(view: &PendingTtlPolicyBudgetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_score + (parsed_window_days as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_ttl_policy_budget("code-409", "label-899");
        let view = to_pending_ttl_policy_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_ttl_policy_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
