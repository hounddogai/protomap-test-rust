use super::*;

use acme_kit::text;

/// How screens and exports show a limit checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitCheckpointView {
    pub code: String,
    pub title_text: String,
    pub limit: String,
    pub summary: String,
    pub retries: String,
    pub rotation_days: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_limit_checkpoint_view(item: &LimitCheckpoint) -> LimitCheckpointView {
    LimitCheckpointView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 120),
        limit: item.limit.to_string(),
        summary: item.summary.clone(),
        retries: item.retries.to_string(),
        rotation_days: item.rotation_days.to_string(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_checkpoint_views(items: &[LimitCheckpoint]) -> Vec<LimitCheckpointView> {
    let mut views: Vec<LimitCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_checkpoint_view(item));
    }
    views
}

pub fn sum_limit_checkpoint_view(view: &LimitCheckpointView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_retries as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_checkpoint("code-993", "title-940");
        let view = to_limit_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
