use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal timeline digest.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineDigestView {
    pub code: String,
    pub name_text: String,
    pub version_text: String,
    pub attempts: String,
    pub label: String,
    pub summary_text: String,
    pub namespace: String,
    pub amount: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_seasonal_timeline_digest_view(item: &SeasonalTimelineDigest) -> SeasonalTimelineDigestView {
    SeasonalTimelineDigestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        version_text: item.version.to_string(),
        attempts: item.attempts.to_string(),
        label: item.label.clone(),
        summary_text: item.summary.clone(),
        namespace: text::truncate(&item.namespace, 64),
        amount: format!("{:.2}", item.amount),
        title_text: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_timeline_digest_views(items: &[SeasonalTimelineDigest]) -> Vec<SeasonalTimelineDigestView> {
    let mut views: Vec<SeasonalTimelineDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_timeline_digest_view(item));
    }
    views
}

pub fn sum_seasonal_timeline_digest_view(view: &SeasonalTimelineDigestView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_version as f64 + (parsed_attempts as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_timeline_digest("code-879", "name-25");
        let view = to_seasonal_timeline_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_timeline_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
