use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemClaimView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub margin_text: String,
    pub grace_days: String,
    pub region_text: String,
    pub quantity_text: String,
    pub retries: String,
    pub description: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_postmortem_claim_view(item: &PostmortemClaim) -> PostmortemClaimView {
    PostmortemClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        margin_text: format!("{:.2}", item.margin),
        grace_days: item.grace_days.to_string(),
        region_text: item.region.clone(),
        quantity_text: item.quantity.to_string(),
        retries: item.retries.to_string(),
        description: item.description.clone(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_claim_views(items: &[PostmortemClaim]) -> Vec<PostmortemClaimView> {
    let mut views: Vec<PostmortemClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_claim_view(item));
    }
    views
}

pub fn sum_postmortem_claim_view(view: &PostmortemClaimView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_cost + parsed_margin + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_claim("code-949", "label-988");
        let view = to_postmortem_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
