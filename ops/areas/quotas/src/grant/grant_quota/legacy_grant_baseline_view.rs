use super::*;

use acme_kit::text;

/// How screens and exports show a legacy grant baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineView {
    pub code: String,
    pub name_text: String,
    pub rate: String,
    pub port: String,
    pub maximum: String,
    pub amount_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_legacy_grant_baseline_view(item: &LegacyGrantBaseline) -> LegacyGrantBaselineView {
    LegacyGrantBaselineView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        rate: format!("{:.2}", item.rate),
        port: item.port.to_string(),
        maximum: format!("{:.2}", item.maximum),
        amount_text: format!("{:.2}", item.amount),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_grant_baseline_views(items: &[LegacyGrantBaseline]) -> Vec<LegacyGrantBaselineView> {
    let mut views: Vec<LegacyGrantBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_grant_baseline_view(item));
    }
    views
}

pub fn sum_legacy_grant_baseline_view(view: &LegacyGrantBaselineView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + (parsed_port as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_grant_baseline("code-496", "name-703");
        let view = to_legacy_grant_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_grant_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
