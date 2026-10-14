use super::*;

use acme_kit::text;

/// How screens and exports show a manual failover baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverBaselineView {
    pub code: String,
    pub label: String,
    pub ceiling: String,
    pub ratio_text: String,
    pub rate: String,
    pub rotation_days: String,
    pub batch_size: String,
    pub locale: String,
    pub host: String,
    pub timezone: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_manual_failover_baseline_view(item: &ManualFailoverBaseline) -> ManualFailoverBaselineView {
    ManualFailoverBaselineView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        ceiling: format!("{:.2}", item.ceiling),
        ratio_text: format!("{:.2}", item.ratio),
        rate: format!("{:.2}", item.rate),
        rotation_days: item.rotation_days.to_string(),
        batch_size: item.batch_size.to_string(),
        locale: item.locale.clone(),
        host: item.host.clone(),
        timezone: item.timezone.clone(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_failover_baseline_views(items: &[ManualFailoverBaseline]) -> Vec<ManualFailoverBaselineView> {
    let mut views: Vec<ManualFailoverBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_failover_baseline_view(item));
    }
    views
}

pub fn sum_manual_failover_baseline_view(view: &ManualFailoverBaselineView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ceiling + parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_failover_baseline("code-122", "label-15");
        let view = to_manual_failover_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_failover_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
