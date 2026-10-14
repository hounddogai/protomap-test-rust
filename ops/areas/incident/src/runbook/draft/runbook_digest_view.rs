use super::*;

use acme_kit::text;

/// How screens and exports show a runbook digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDigestView {
    pub code_text: String,
    pub title_text: String,
    pub duration_minutes: String,
    pub currency: String,
    pub score: String,
    pub threshold_text: String,
    pub visible_text: String,
    pub audience: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_runbook_digest_view(item: &RunbookDigest) -> RunbookDigestView {
    RunbookDigestView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 80),
        duration_minutes: item.duration_minutes.to_string(),
        currency: item.currency.clone(),
        score: format!("{:.2}", item.score),
        threshold_text: format!("{:.2}", item.threshold),
        visible_text: format!("{}", item.visible),
        audience: item.audience.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_digest_views(items: &[RunbookDigest]) -> Vec<RunbookDigestView> {
    let mut views: Vec<RunbookDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_digest_view(item));
    }
    views
}

pub fn sum_runbook_digest_view(view: &RunbookDigestView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_score + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_digest("code-426", "title-481");
        let view = to_runbook_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
