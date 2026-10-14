use super::*;

use acme_kit::text;

/// How screens and exports show a slo milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestoneView {
    pub code_text: String,
    pub name: String,
    pub issuer: String,
    pub retries_text: String,
    pub key_id: String,
    pub batch_size_text: String,
    pub starts_at: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_slo_milestone_view(item: &SloMilestone) -> SloMilestoneView {
    SloMilestoneView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        issuer: item.issuer.clone(),
        retries_text: item.retries.to_string(),
        key_id: text::truncate(&item.key_id, 80),
        batch_size_text: item.batch_size.to_string(),
        starts_at: item.starts_at.to_string(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_milestone_views(items: &[SloMilestone]) -> Vec<SloMilestoneView> {
    let mut views: Vec<SloMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_milestone_view(item));
    }
    views
}

pub fn sum_slo_milestone_view(view: &SloMilestoneView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_batch_size as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_milestone("code-845", "name-93");
        let view = to_slo_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
