use super::*;

use acme_kit::text;

/// How screens and exports show a alert rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollupView {
    pub code_text: String,
    pub title: String,
    pub currency: String,
    pub summary_text: String,
    pub contact_email: String,
    pub principal: String,
    pub sequence: String,
    pub expires_at: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_alert_rollup_view(item: &AlertRollup) -> AlertRollupView {
    AlertRollupView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        currency: text::truncate(&item.currency, 64),
        summary_text: item.summary.clone(),
        contact_email: text::truncate(&item.contact_email, 64),
        principal: text::truncate(&item.principal, 120),
        sequence: item.sequence.to_string(),
        expires_at: item.expires_at.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_rollup_views(items: &[AlertRollup]) -> Vec<AlertRollupView> {
    let mut views: Vec<AlertRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_rollup_view(item));
    }
    views
}

pub fn sum_alert_rollup_view(view: &AlertRollupView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_expires_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_rollup("code-930", "title-328");
        let view = to_alert_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
