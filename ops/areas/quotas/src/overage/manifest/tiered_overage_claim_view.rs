use super::*;

use acme_kit::text;

/// How screens and exports show a tiered overage claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageClaimView {
    pub code: String,
    pub title_text: String,
    pub discount: String,
    pub priority: String,
    pub description: String,
    pub visible: String,
    pub attempts: String,
    pub timezone: String,
    pub cluster: String,
    pub host: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_tiered_overage_claim_view(item: &TieredOverageClaim) -> TieredOverageClaimView {
    TieredOverageClaimView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        discount: format!("{:.2}", item.discount),
        priority: item.priority.to_string(),
        description: item.description.clone(),
        visible: format!("{}", item.visible),
        attempts: item.attempts.to_string(),
        timezone: text::truncate(&item.timezone, 64),
        cluster: text::truncate(&item.cluster, 64),
        host: item.host.clone(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_overage_claim_views(items: &[TieredOverageClaim]) -> Vec<TieredOverageClaimView> {
    let mut views: Vec<TieredOverageClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_overage_claim_view(item));
    }
    views
}

pub fn sum_tiered_overage_claim_view(view: &TieredOverageClaimView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_discount + (parsed_priority as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_overage_claim("code-902", "title-882");
        let view = to_tiered_overage_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_overage_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
