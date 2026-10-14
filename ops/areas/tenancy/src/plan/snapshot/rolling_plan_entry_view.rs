use super::*;

use acme_kit::text;

/// How screens and exports show a rolling plan entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanEntryView {
    pub code: String,
    pub name_text: String,
    pub external_ref_text: String,
    pub discount: String,
    pub burst: String,
    pub source: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_rolling_plan_entry_view(item: &RollingPlanEntry) -> RollingPlanEntryView {
    RollingPlanEntryView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 80),
        external_ref_text: item.external_ref.clone(),
        discount: format!("{:.2}", item.discount),
        burst: item.burst.to_string(),
        source: item.source.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_plan_entry_views(items: &[RollingPlanEntry]) -> Vec<RollingPlanEntryView> {
    let mut views: Vec<RollingPlanEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_plan_entry_view(item));
    }
    views
}

pub fn sum_rolling_plan_entry_view(view: &RollingPlanEntryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_discount + (parsed_burst as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_plan_entry("code-287", "name-711");
        let view = to_rolling_plan_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_plan_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
