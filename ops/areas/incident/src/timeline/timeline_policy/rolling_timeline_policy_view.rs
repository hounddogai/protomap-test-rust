use super::*;

use acme_kit::text;

/// How screens and exports show a rolling timeline policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicyView {
    pub code_text: String,
    pub title_text: String,
    pub reviewer_text: String,
    pub summary_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_rolling_timeline_policy_view(item: &RollingTimelinePolicy) -> RollingTimelinePolicyView {
    RollingTimelinePolicyView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        reviewer_text: item.reviewer.clone(),
        summary_text: text::truncate(&item.summary, 80),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_timeline_policy_views(items: &[RollingTimelinePolicy]) -> Vec<RollingTimelinePolicyView> {
    let mut views: Vec<RollingTimelinePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_timeline_policy_view(item));
    }
    views
}

pub fn sum_rolling_timeline_policy_view(view: &RollingTimelinePolicyView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_timeline_policy("code-106", "title-672");
        let view = to_rolling_timeline_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_timeline_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
