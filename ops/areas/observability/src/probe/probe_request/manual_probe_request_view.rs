use super::*;

use acme_kit::text;

/// How screens and exports show a manual probe request.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestView {
    pub code: String,
    pub name_text: String,
    pub title: String,
    pub fingerprint: String,
    pub source_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_manual_probe_request_view(item: &ManualProbeRequest) -> ManualProbeRequestView {
    ManualProbeRequestView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        source_text: text::truncate(&item.source, 40),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_probe_request_views(items: &[ManualProbeRequest]) -> Vec<ManualProbeRequestView> {
    let mut views: Vec<ManualProbeRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_probe_request_view(item));
    }
    views
}

pub fn sum_manual_probe_request_view(view: &ManualProbeRequestView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_probe_request("code-551", "name-440");
        let view = to_manual_probe_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_probe_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
