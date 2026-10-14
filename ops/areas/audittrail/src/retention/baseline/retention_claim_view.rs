use super::*;

use acme_kit::text;

/// How screens and exports show a retention claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionClaimView {
    pub code: String,
    pub title_text: String,
    pub audience_text: String,
    pub floor_value: String,
    pub name_text: String,
    pub rotation_days_text: String,
    pub currency: String,
    pub expires_at_text: String,
    pub summary: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_retention_claim_view(item: &RetentionClaim) -> RetentionClaimView {
    RetentionClaimView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        audience_text: text::truncate(&item.audience, 80),
        floor_value: format!("{:.2}", item.floor_value),
        name_text: text::truncate(&item.name, 120),
        rotation_days_text: item.rotation_days.to_string(),
        currency: item.currency.clone(),
        expires_at_text: item.expires_at.to_string(),
        summary: item.summary.clone(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_claim_views(items: &[RetentionClaim]) -> Vec<RetentionClaimView> {
    let mut views: Vec<RetentionClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_claim_view(item));
    }
    views
}

pub fn sum_retention_claim_view(view: &RetentionClaimView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_floor_value + (parsed_rotation_days as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_claim("code-338", "title-994");
        let view = to_retention_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
