use super::*;

use acme_kit::text;

/// How screens and exports show a overage checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageCheckpointView {
    pub code: String,
    pub label: String,
    pub name: String,
    pub retries: String,
    pub approved: String,
    pub minimum: String,
    pub duration_minutes_text: String,
    pub owner_id_text: String,
    pub audience_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_overage_checkpoint_view(item: &OverageCheckpoint) -> OverageCheckpointView {
    OverageCheckpointView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        approved: format!("{}", item.approved),
        minimum: format!("{:.2}", item.minimum),
        duration_minutes_text: item.duration_minutes.to_string(),
        owner_id_text: item.owner_id.clone(),
        audience_text: item.audience.clone(),
        locale: text::truncate(&item.locale, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_checkpoint_views(items: &[OverageCheckpoint]) -> Vec<OverageCheckpointView> {
    let mut views: Vec<OverageCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_checkpoint_view(item));
    }
    views
}

pub fn sum_overage_checkpoint_view(view: &OverageCheckpointView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_retries as f64 + parsed_minimum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_checkpoint("code-531", "label-913");
        let view = to_overage_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
