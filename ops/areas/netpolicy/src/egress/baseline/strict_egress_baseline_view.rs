use super::*;

use acme_kit::text;

/// How screens and exports show a strict egress baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBaselineView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub ratio: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_strict_egress_baseline_view(item: &StrictEgressBaseline) -> StrictEgressBaselineView {
    StrictEgressBaselineView {
        code: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 64),
        pinned: format!("{}", item.pinned),
        ratio: format!("{:.2}", item.ratio),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_egress_baseline_views(items: &[StrictEgressBaseline]) -> Vec<StrictEgressBaselineView> {
    let mut views: Vec<StrictEgressBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_egress_baseline_view(item));
    }
    views
}

pub fn sum_strict_egress_baseline_view(view: &StrictEgressBaselineView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ratio + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_egress_baseline("code-252", "label-160");
        let view = to_strict_egress_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_egress_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
