use super::*;

use acme_kit::text;

/// How screens and exports show a peering checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringCheckpointView {
    pub code_text: String,
    pub title: String,
    pub burst_text: String,
    pub label: String,
    pub floor_value: String,
    pub key_id_text: String,
    pub host: String,
    pub country: String,
    pub grace_days: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_peering_checkpoint_view(item: &PeeringCheckpoint) -> PeeringCheckpointView {
    PeeringCheckpointView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        burst_text: item.burst.to_string(),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        key_id_text: item.key_id.clone(),
        host: text::truncate(&item.host, 120),
        country: item.country.clone(),
        grace_days: item.grace_days.to_string(),
        cipher: text::truncate(&item.cipher, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_checkpoint_views(items: &[PeeringCheckpoint]) -> Vec<PeeringCheckpointView> {
    let mut views: Vec<PeeringCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_checkpoint_view(item));
    }
    views
}

pub fn sum_peering_checkpoint_view(view: &PeeringCheckpointView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_burst as f64 + parsed_floor_value + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_checkpoint("code-263", "title-849");
        let view = to_peering_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
