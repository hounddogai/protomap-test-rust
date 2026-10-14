use super::*;

use acme_kit::text;

/// How screens and exports show a legacy isolation record.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub region: String,
    pub contact_email: String,
    pub maximum: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_legacy_isolation_record_view(item: &LegacyIsolationRecord) -> LegacyIsolationRecordView {
    LegacyIsolationRecordView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        priority: item.priority.to_string(),
        region: item.region.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        maximum: format!("{:.2}", item.maximum),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_isolation_record_views(items: &[LegacyIsolationRecord]) -> Vec<LegacyIsolationRecordView> {
    let mut views: Vec<LegacyIsolationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_isolation_record_view(item));
    }
    views
}

pub fn sum_legacy_isolation_record_view(view: &LegacyIsolationRecordView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_priority as f64 + parsed_maximum + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_isolation_record("code-232", "title-58");
        let view = to_legacy_isolation_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_isolation_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
