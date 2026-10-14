use super::*;

use acme_kit::text;

/// How screens and exports show a legacy redaction score.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScoreView {
    pub code: String,
    pub label: String,
    pub expires_at: String,
    pub threshold: String,
    pub summary: String,
    pub duration_minutes: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_legacy_redaction_score_view(item: &LegacyRedactionScore) -> LegacyRedactionScoreView {
    LegacyRedactionScoreView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        expires_at: item.expires_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        summary: text::truncate(&item.summary, 40),
        duration_minutes: item.duration_minutes.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_redaction_score_views(items: &[LegacyRedactionScore]) -> Vec<LegacyRedactionScoreView> {
    let mut views: Vec<LegacyRedactionScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_redaction_score_view(item));
    }
    views
}

pub fn sum_legacy_redaction_score_view(view: &LegacyRedactionScoreView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_threshold + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_redaction_score("code-893", "label-740");
        let view = to_legacy_redaction_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_redaction_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
