use super::*;

use acme_kit::text;

/// How screens and exports show a runbook claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaimView {
    pub code: String,
    pub label: String,
    pub channel_text: String,
    pub version: String,
    pub window_days_text: String,
    pub revision: String,
    pub segment: String,
    pub timezone_text: String,
    pub weight_text: String,
    pub currency_text: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_runbook_claim_view(item: &RunbookClaim) -> RunbookClaimView {
    RunbookClaimView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        channel_text: text::truncate(&item.channel, 64),
        version: item.version.to_string(),
        window_days_text: item.window_days.to_string(),
        revision: item.revision.to_string(),
        segment: item.segment.clone(),
        timezone_text: item.timezone.clone(),
        weight_text: format!("{:.2}", item.weight),
        currency_text: text::truncate(&item.currency, 120),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_claim_views(items: &[RunbookClaim]) -> Vec<RunbookClaimView> {
    let mut views: Vec<RunbookClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_claim_view(item));
    }
    views
}

pub fn sum_runbook_claim_view(view: &RunbookClaimView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_version as f64 + (parsed_window_days as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_claim("code-662", "label-42");
        let view = to_runbook_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
