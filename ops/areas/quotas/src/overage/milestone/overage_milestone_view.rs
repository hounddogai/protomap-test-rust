use super::*;

use acme_kit::text;

/// How screens and exports show a overage milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageMilestoneView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub limit: String,
    pub source_text: String,
    pub starts_at: String,
    pub unit_text: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_overage_milestone_view(item: &OverageMilestone) -> OverageMilestoneView {
    OverageMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        limit: item.limit.to_string(),
        source_text: item.source.clone(),
        starts_at: item.starts_at.to_string(),
        unit_text: text::truncate(&item.unit, 64),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_milestone_views(items: &[OverageMilestone]) -> Vec<OverageMilestoneView> {
    let mut views: Vec<OverageMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_milestone_view(item));
    }
    views
}

pub fn sum_overage_milestone_view(view: &OverageMilestoneView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_discount + (parsed_limit as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_milestone("code-424", "label-376");
        let view = to_overage_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
