use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal rollback claim.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaimView {
    pub code: String,
    pub name: String,
    pub pinned: String,
    pub expires_at: String,
    pub contact_email: String,
    pub key_id: String,
    pub ends_at: String,
    pub ttl_seconds: String,
    pub updated_at: String,
    pub country: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_seasonal_rollback_claim_view(item: &SeasonalRollbackClaim) -> SeasonalRollbackClaimView {
    SeasonalRollbackClaimView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        pinned: format!("{}", item.pinned),
        expires_at: item.expires_at.to_string(),
        contact_email: item.contact_email.clone(),
        key_id: item.key_id.clone(),
        ends_at: item.ends_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        updated_at: item.updated_at.to_string(),
        country: item.country.clone(),
        cluster: text::truncate(&item.cluster, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_rollback_claim_views(items: &[SeasonalRollbackClaim]) -> Vec<SeasonalRollbackClaimView> {
    let mut views: Vec<SeasonalRollbackClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_rollback_claim_view(item));
    }
    views
}

pub fn sum_seasonal_rollback_claim_view(view: &SeasonalRollbackClaimView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_ends_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_rollback_claim("code-262", "name-202");
        let view = to_seasonal_rollback_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_rollback_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
