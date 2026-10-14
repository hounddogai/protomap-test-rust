use super::*;

use acme_kit::text;

/// How screens and exports show a plan claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanClaimView {
    pub code_text: String,
    pub name: String,
    pub retries: String,
    pub expires_at_text: String,
    pub region_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_plan_claim_view(item: &PlanClaim) -> PlanClaimView {
    PlanClaimView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        expires_at_text: item.expires_at.to_string(),
        region_text: text::truncate(&item.region, 64),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_claim_views(items: &[PlanClaim]) -> Vec<PlanClaimView> {
    let mut views: Vec<PlanClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_claim_view(item));
    }
    views
}

pub fn sum_plan_claim_view(view: &PlanClaimView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_expires_at as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_claim("code-45", "name-109");
        let view = to_plan_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
