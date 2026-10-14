use super::*;

use acme_kit::text;

/// How screens and exports show a canary summary.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummaryView {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub cluster_text: String,
    pub external_ref_text: String,
    pub threshold: String,
    pub key_id: String,
    pub maximum: String,
    pub unit: String,
    pub contact_email: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_canary_summary_view(item: &CanarySummary) -> CanarySummaryView {
    CanarySummaryView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        channel: item.channel.clone(),
        cluster_text: item.cluster.clone(),
        external_ref_text: item.external_ref.clone(),
        threshold: format!("{:.2}", item.threshold),
        key_id: item.key_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        unit: item.unit.clone(),
        contact_email: item.contact_email.clone(),
        host_text: text::truncate(&item.host, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_summary_views(items: &[CanarySummary]) -> Vec<CanarySummaryView> {
    let mut views: Vec<CanarySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_summary_view(item));
    }
    views
}

pub fn sum_canary_summary_view(view: &CanarySummaryView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_threshold + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_summary("code-317", "name-33");
        let view = to_canary_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
