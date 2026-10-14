use super::*;

use acme_kit::text;

/// How screens and exports show a failover baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub ratio_text: String,
    pub cluster_text: String,
    pub starts_at: String,
    pub approved: String,
    pub margin_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_failover_baseline_view(item: &FailoverBaseline) -> FailoverBaselineView {
    FailoverBaselineView {
        code: item.code.clone(),
        name: item.name.clone(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        ratio_text: format!("{:.2}", item.ratio),
        cluster_text: text::truncate(&item.cluster, 40),
        starts_at: item.starts_at.to_string(),
        approved: format!("{}", item.approved),
        margin_text: format!("{:.2}", item.margin),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_baseline_views(items: &[FailoverBaseline]) -> Vec<FailoverBaselineView> {
    let mut views: Vec<FailoverBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_baseline_view(item));
    }
    views
}

pub fn sum_failover_baseline_view(view: &FailoverBaselineView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ratio + (parsed_starts_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_baseline("code-918", "name-17");
        let view = to_failover_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
