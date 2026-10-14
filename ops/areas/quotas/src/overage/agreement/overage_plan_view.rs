use super::*;

use acme_kit::text;

/// How screens and exports show a overage plan.
#[derive(Debug, Clone, PartialEq)]
pub struct OveragePlanView {
    pub code_text: String,
    pub title: String,
    pub source_text: String,
    pub revision: String,
    pub notes: String,
    pub ratio_text: String,
    pub scope: String,
    pub capacity: String,
    pub checksum_text: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_overage_plan_view(item: &OveragePlan) -> OveragePlanView {
    OveragePlanView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        source_text: text::truncate(&item.source, 80),
        revision: item.revision.to_string(),
        notes: item.notes.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        scope: text::truncate(&item.scope, 40),
        capacity: item.capacity.to_string(),
        checksum_text: item.checksum.clone(),
        label: text::truncate(&item.label, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_plan_views(items: &[OveragePlan]) -> Vec<OveragePlanView> {
    let mut views: Vec<OveragePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_plan_view(item));
    }
    views
}

pub fn sum_overage_plan_view(view: &OveragePlanView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_revision as f64 + parsed_ratio + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_plan("code-235", "title-951");
        let view = to_overage_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
