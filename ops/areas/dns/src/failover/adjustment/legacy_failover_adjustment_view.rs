use super::*;

use acme_kit::text;

/// How screens and exports show a legacy failover adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverAdjustmentView {
    pub code: String,
    pub name_text: String,
    pub sequence: String,
    pub ceiling: String,
    pub region: String,
    pub description_text: String,
    pub duration_minutes_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_legacy_failover_adjustment_view(item: &LegacyFailoverAdjustment) -> LegacyFailoverAdjustmentView {
    LegacyFailoverAdjustmentView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        sequence: item.sequence.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        region: text::truncate(&item.region, 80),
        description_text: item.description.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_failover_adjustment_views(items: &[LegacyFailoverAdjustment]) -> Vec<LegacyFailoverAdjustmentView> {
    let mut views: Vec<LegacyFailoverAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_failover_adjustment_view(item));
    }
    views
}

pub fn sum_legacy_failover_adjustment_view(view: &LegacyFailoverAdjustmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_ceiling + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_failover_adjustment("code-418", "name-399");
        let view = to_legacy_failover_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_failover_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
