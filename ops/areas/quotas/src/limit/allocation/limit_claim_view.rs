use super::*;

use acme_kit::text;

/// How screens and exports show a limit claim.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitClaimView {
    pub code: String,
    pub title_text: String,
    pub approved: String,
    pub margin: String,
    pub category: String,
    pub region_text: String,
    pub amount: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_limit_claim_view(item: &LimitClaim) -> LimitClaimView {
    LimitClaimView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        approved: format!("{}", item.approved),
        margin: format!("{:.2}", item.margin),
        category: item.category.clone(),
        region_text: item.region.clone(),
        amount: format!("{:.2}", item.amount),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_claim_views(items: &[LimitClaim]) -> Vec<LimitClaimView> {
    let mut views: Vec<LimitClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_claim_view(item));
    }
    views
}

pub fn sum_limit_claim_view(view: &LimitClaimView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_margin + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_claim("code-465", "title-785");
        let view = to_limit_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
