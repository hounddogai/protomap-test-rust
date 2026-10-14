use super::*;

use acme_kit::text;

/// How screens and exports show a strict evidence budget.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetView {
    pub code: String,
    pub title: String,
    pub rate_text: String,
    pub cluster: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_strict_evidence_budget_view(item: &StrictEvidenceBudget) -> StrictEvidenceBudgetView {
    StrictEvidenceBudgetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        rate_text: format!("{:.2}", item.rate),
        cluster: item.cluster.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_evidence_budget_views(items: &[StrictEvidenceBudget]) -> Vec<StrictEvidenceBudgetView> {
    let mut views: Vec<StrictEvidenceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_evidence_budget_view(item));
    }
    views
}

pub fn sum_strict_evidence_budget_view(view: &StrictEvidenceBudgetView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_rate + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_evidence_budget("code-916", "title-295");
        let view = to_strict_evidence_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_evidence_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
