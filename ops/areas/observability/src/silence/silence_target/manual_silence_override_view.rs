use super::*;

use acme_kit::text;

/// How screens and exports show a manual silence override.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverrideView {
    pub code: String,
    pub name_text: String,
    pub cluster: String,
    pub category: String,
    pub key_id: String,
    pub summary: String,
    pub notes: String,
    pub cost_text: String,
    pub ttl_seconds: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_manual_silence_override_view(item: &ManualSilenceOverride) -> ManualSilenceOverrideView {
    ManualSilenceOverrideView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        cluster: item.cluster.clone(),
        category: item.category.clone(),
        key_id: text::truncate(&item.key_id, 40),
        summary: item.summary.clone(),
        notes: text::truncate(&item.notes, 40),
        cost_text: format!("{:.2}", item.cost),
        ttl_seconds: item.ttl_seconds.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_silence_override_views(items: &[ManualSilenceOverride]) -> Vec<ManualSilenceOverrideView> {
    let mut views: Vec<ManualSilenceOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_silence_override_view(item));
    }
    views
}

pub fn sum_manual_silence_override_view(view: &ManualSilenceOverrideView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_cost + (parsed_ttl_seconds as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_silence_override("code-943", "name-891");
        let view = to_manual_silence_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_silence_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
