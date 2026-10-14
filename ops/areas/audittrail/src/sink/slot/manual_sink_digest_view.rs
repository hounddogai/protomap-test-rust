use super::*;

use acme_kit::text;

/// How screens and exports show a manual sink digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestView {
    pub code_text: String,
    pub label: String,
    pub updated_at: String,
    pub phone: String,
    pub grace_days_text: String,
    pub cluster: String,
    pub archived: String,
    pub created_at: String,
    pub fingerprint: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_manual_sink_digest_view(item: &ManualSinkDigest) -> ManualSinkDigestView {
    ManualSinkDigestView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        phone: text::truncate(&item.phone, 120),
        grace_days_text: item.grace_days.to_string(),
        cluster: item.cluster.clone(),
        archived: format!("{}", item.archived),
        created_at: item.created_at.to_string(),
        fingerprint: item.fingerprint.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_sink_digest_views(items: &[ManualSinkDigest]) -> Vec<ManualSinkDigestView> {
    let mut views: Vec<ManualSinkDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_sink_digest_view(item));
    }
    views
}

pub fn sum_manual_sink_digest_view(view: &ManualSinkDigestView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_grace_days as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_sink_digest("code-862", "label-902");
        let view = to_manual_sink_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_sink_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
