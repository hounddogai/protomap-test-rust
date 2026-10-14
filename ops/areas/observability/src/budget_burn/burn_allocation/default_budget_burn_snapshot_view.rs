use super::*;

/// How screens and exports show a default budget burn snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnSnapshotView {
    pub code_text: String,
    pub title_text: String,
    pub score: String,
    pub attempts: String,
    pub label_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_default_budget_burn_snapshot_view(item: &DefaultBudgetBurnSnapshot) -> DefaultBudgetBurnSnapshotView {
    DefaultBudgetBurnSnapshotView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        score: format!("{:.2}", item.score),
        attempts: item.attempts.to_string(),
        label_text: item.label.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_budget_burn_snapshot_views(
    items: &[DefaultBudgetBurnSnapshot],
) -> Vec<DefaultBudgetBurnSnapshotView> {
    let mut views: Vec<DefaultBudgetBurnSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_budget_burn_snapshot_view(item));
    }
    views
}

pub fn sum_default_budget_burn_snapshot_view(view: &DefaultBudgetBurnSnapshotView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_score + (parsed_attempts as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_budget_burn_snapshot("code-486", "title-181");
        let view = to_default_budget_burn_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_budget_burn_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
