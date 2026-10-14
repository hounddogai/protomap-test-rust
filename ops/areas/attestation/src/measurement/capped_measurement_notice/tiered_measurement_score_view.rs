use super::*;

use acme_kit::text;

/// How screens and exports show a tiered measurement score.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreView {
    pub code: String,
    pub name: String,
    pub segment_text: String,
    pub active: String,
    pub title: String,
    pub revision: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_tiered_measurement_score_view(item: &TieredMeasurementScore) -> TieredMeasurementScoreView {
    TieredMeasurementScoreView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        segment_text: item.segment.clone(),
        active: format!("{}", item.active),
        title: item.title.clone(),
        revision: item.revision.to_string(),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_measurement_score_views(items: &[TieredMeasurementScore]) -> Vec<TieredMeasurementScoreView> {
    let mut views: Vec<TieredMeasurementScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_measurement_score_view(item));
    }
    views
}

pub fn sum_tiered_measurement_score_view(view: &TieredMeasurementScoreView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_measurement_score("code-37", "name-308");
        let view = to_tiered_measurement_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_measurement_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
