use super::*;

use acme_kit::text;

/// How screens and exports show a pending rule record.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecordView {
    pub code: String,
    pub title_text: String,
    pub category: String,
    pub currency_text: String,
    pub checksum_text: String,
    pub duration_minutes_text: String,
    pub unit_text: String,
    pub label_text: String,
    pub sequence: String,
    pub retries_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_pending_rule_record_view(item: &PendingRuleRecord) -> PendingRuleRecordView {
    PendingRuleRecordView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        category: item.category.clone(),
        currency_text: item.currency.clone(),
        checksum_text: text::truncate(&item.checksum, 40),
        duration_minutes_text: item.duration_minutes.to_string(),
        unit_text: text::truncate(&item.unit, 40),
        label_text: text::truncate(&item.label, 64),
        sequence: item.sequence.to_string(),
        retries_text: item.retries.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_rule_record_views(items: &[PendingRuleRecord]) -> Vec<PendingRuleRecordView> {
    let mut views: Vec<PendingRuleRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_rule_record_view(item));
    }
    views
}

pub fn sum_pending_rule_record_view(view: &PendingRuleRecordView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_sequence as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_rule_record("code-241", "title-45");
        let view = to_pending_rule_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_rule_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
