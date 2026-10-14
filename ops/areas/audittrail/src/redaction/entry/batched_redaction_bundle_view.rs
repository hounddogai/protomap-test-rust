use super::*;

use acme_kit::text;

/// How screens and exports show a batched redaction bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionBundleView {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub port_text: String,
    pub created_at: String,
    pub key_id: String,
    pub window_days: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_batched_redaction_bundle_view(item: &BatchedRedactionBundle) -> BatchedRedactionBundleView {
    BatchedRedactionBundleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        reviewer: item.reviewer.clone(),
        port_text: item.port.to_string(),
        created_at: item.created_at.to_string(),
        key_id: text::truncate(&item.key_id, 40),
        window_days: item.window_days.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_redaction_bundle_views(items: &[BatchedRedactionBundle]) -> Vec<BatchedRedactionBundleView> {
    let mut views: Vec<BatchedRedactionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_redaction_bundle_view(item));
    }
    views
}

pub fn sum_batched_redaction_bundle_view(view: &BatchedRedactionBundleView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_port as f64 + (parsed_created_at as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_redaction_bundle("code-73", "name-115");
        let view = to_batched_redaction_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_redaction_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
