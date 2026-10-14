use super::*;

use acme_kit::text;

/// How screens and exports show a primary rollout rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollupView {
    pub code_text: String,
    pub title: String,
    pub visible: String,
    pub created_at: String,
    pub rate_text: String,
    pub reviewer_text: String,
    pub rotation_days: String,
    pub unit_text: String,
    pub key_id: String,
    pub starts_at_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_primary_rollout_rollup_view(item: &PrimaryRolloutRollup) -> PrimaryRolloutRollupView {
    PrimaryRolloutRollupView {
        code_text: text::truncate(&item.code, 80),
        title: item.title.clone(),
        visible: format!("{}", item.visible),
        created_at: item.created_at.to_string(),
        rate_text: format!("{:.2}", item.rate),
        reviewer_text: text::truncate(&item.reviewer, 80),
        rotation_days: item.rotation_days.to_string(),
        unit_text: text::truncate(&item.unit, 40),
        key_id: item.key_id.clone(),
        starts_at_text: item.starts_at.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rollout_rollup_views(items: &[PrimaryRolloutRollup]) -> Vec<PrimaryRolloutRollupView> {
    let mut views: Vec<PrimaryRolloutRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rollout_rollup_view(item));
    }
    views
}

pub fn sum_primary_rollout_rollup_view(view: &PrimaryRolloutRollupView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_rate + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rollout_rollup("code-86", "title-826");
        let view = to_primary_rollout_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rollout_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
