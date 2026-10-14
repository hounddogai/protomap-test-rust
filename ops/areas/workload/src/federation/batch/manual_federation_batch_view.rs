use super::*;

use acme_kit::text;

/// How screens and exports show a manual federation batch.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatchView {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub contact_email_text: String,
    pub scope_text: String,
    pub namespace_text: String,
    pub phone: String,
    pub port_text: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_manual_federation_batch_view(item: &ManualFederationBatch) -> ManualFederationBatchView {
    ManualFederationBatchView {
        code: item.code.clone(),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        contact_email_text: item.contact_email.clone(),
        scope_text: text::truncate(&item.scope, 40),
        namespace_text: item.namespace.clone(),
        phone: text::truncate(&item.phone, 64),
        port_text: item.port.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_federation_batch_views(items: &[ManualFederationBatch]) -> Vec<ManualFederationBatchView> {
    let mut views: Vec<ManualFederationBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_federation_batch_view(item));
    }
    views
}

pub fn sum_manual_federation_batch_view(view: &ManualFederationBatchView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_federation_batch("code-610", "label-119");
        let view = to_manual_federation_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_federation_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
