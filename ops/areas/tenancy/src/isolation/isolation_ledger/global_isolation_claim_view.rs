use super::*;

use acme_kit::text;

/// How screens and exports show a global isolation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimView {
    pub code: String,
    pub name_text: String,
    pub rotation_days: String,
    pub attempts: String,
    pub issuer: String,
    pub created_at: String,
    pub threshold: String,
    pub starts_at: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_global_isolation_claim_view(item: &GlobalIsolationClaim) -> GlobalIsolationClaimView {
    GlobalIsolationClaimView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        rotation_days: item.rotation_days.to_string(),
        attempts: item.attempts.to_string(),
        issuer: item.issuer.clone(),
        created_at: item.created_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        starts_at: item.starts_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_isolation_claim_views(items: &[GlobalIsolationClaim]) -> Vec<GlobalIsolationClaimView> {
    let mut views: Vec<GlobalIsolationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_isolation_claim_view(item));
    }
    views
}

pub fn sum_global_isolation_claim_view(view: &GlobalIsolationClaimView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_attempts as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_isolation_claim("code-220", "name-729");
        let view = to_global_isolation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_isolation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
