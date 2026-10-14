use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn digest.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestView {
    pub code: String,
    pub name_text: String,
    pub ttl_seconds: String,
    pub summary: String,
    pub title: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_budget_burn_digest_view(item: &BudgetBurnDigest) -> BudgetBurnDigestView {
    BudgetBurnDigestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        summary: text::truncate(&item.summary, 64),
        title: text::truncate(&item.title, 120),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_digest_views(items: &[BudgetBurnDigest]) -> Vec<BudgetBurnDigestView> {
    let mut views: Vec<BudgetBurnDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_digest_view(item));
    }
    views
}

pub fn sum_budget_burn_digest_view(view: &BudgetBurnDigestView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_digest("code-288", "name-995");
        let view = to_budget_burn_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
