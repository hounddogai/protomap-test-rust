use super::*;

use acme_kit::text;

/// How screens and exports show a manual probe milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneView {
    pub code: String,
    pub title: String,
    pub maximum_text: String,
    pub updated_at: String,
    pub rate: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_manual_probe_milestone_view(item: &ManualProbeMilestone) -> ManualProbeMilestoneView {
    ManualProbeMilestoneView {
        code: item.code.clone(),
        title: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        updated_at: item.updated_at.to_string(),
        rate: format!("{:.2}", item.rate),
        phone_text: text::truncate(&item.phone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_probe_milestone_views(items: &[ManualProbeMilestone]) -> Vec<ManualProbeMilestoneView> {
    let mut views: Vec<ManualProbeMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_probe_milestone_view(item));
    }
    views
}

pub fn sum_manual_probe_milestone_view(view: &ManualProbeMilestoneView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_maximum + (parsed_updated_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_probe_milestone("code-220", "title-422");
        let view = to_manual_probe_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_probe_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
