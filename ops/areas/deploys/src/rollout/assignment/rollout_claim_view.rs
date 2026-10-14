use super::*;

use acme_kit::text;

/// How screens and exports show a rollout claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutClaimView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub minimum: String,
    pub approved_text: String,
    pub contact_email: String,
    pub ratio: String,
    pub notes_text: String,
    pub margin_text: String,
    pub quantity: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_rollout_claim_view(item: &RolloutClaim) -> RolloutClaimView {
    RolloutClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        minimum: format!("{:.2}", item.minimum),
        approved_text: format!("{}", item.approved),
        contact_email: item.contact_email.clone(),
        ratio: format!("{:.2}", item.ratio),
        notes_text: text::truncate(&item.notes, 120),
        margin_text: format!("{:.2}", item.margin),
        quantity: item.quantity.to_string(),
        scope: text::truncate(&item.scope, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_claim_views(items: &[RolloutClaim]) -> Vec<RolloutClaimView> {
    let mut views: Vec<RolloutClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_claim_view(item));
    }
    views
}

pub fn sum_rollout_claim_view(view: &RolloutClaimView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_priority as f64 + parsed_minimum + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_claim("code-504", "title-603");
        let view = to_rollout_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
