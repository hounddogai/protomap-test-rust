use super::*;

use acme_kit::text;

/// How screens and exports show a weighted budget burn audit.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditView {
    pub code_text: String,
    pub name_text: String,
    pub expires_at: String,
    pub key_id: String,
    pub cipher_text: String,
    pub duration_minutes_text: String,
    pub rate: String,
    pub ceiling: String,
    pub created_at: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_weighted_budget_burn_audit_view(item: &WeightedBudgetBurnAudit) -> WeightedBudgetBurnAuditView {
    WeightedBudgetBurnAuditView {
        code_text: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 80),
        expires_at: item.expires_at.to_string(),
        key_id: item.key_id.clone(),
        cipher_text: item.cipher.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        rate: format!("{:.2}", item.rate),
        ceiling: format!("{:.2}", item.ceiling),
        created_at: item.created_at.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_budget_burn_audit_views(items: &[WeightedBudgetBurnAudit]) -> Vec<WeightedBudgetBurnAuditView> {
    let mut views: Vec<WeightedBudgetBurnAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_budget_burn_audit_view(item));
    }
    views
}

pub fn sum_weighted_budget_burn_audit_view(view: &WeightedBudgetBurnAuditView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_duration_minutes as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_budget_burn_audit("code-59", "name-65");
        let view = to_weighted_budget_burn_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_budget_burn_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
