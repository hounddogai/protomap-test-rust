use super::*;

use acme_kit::text;

/// How screens and exports show a manual verifier budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierBudgetView {
    pub code_text: String,
    pub title: String,
    pub audience: String,
    pub capacity: String,
    pub score: String,
    pub burst_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_manual_verifier_budget_view(item: &ManualVerifierBudget) -> ManualVerifierBudgetView {
    ManualVerifierBudgetView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        audience: item.audience.clone(),
        capacity: item.capacity.to_string(),
        score: format!("{:.2}", item.score),
        burst_text: item.burst.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_verifier_budget_views(items: &[ManualVerifierBudget]) -> Vec<ManualVerifierBudgetView> {
    let mut views: Vec<ManualVerifierBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_verifier_budget_view(item));
    }
    views
}

pub fn sum_manual_verifier_budget_view(view: &ManualVerifierBudgetView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_score + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_verifier_budget("code-709", "title-800");
        let view = to_manual_verifier_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_verifier_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
