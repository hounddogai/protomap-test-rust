use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled allowlist digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistDigestView {
    pub code_text: String,
    pub label_text: String,
    pub audience: String,
    pub quantity_text: String,
    pub created_at: String,
    pub sequence: String,
    pub locale: String,
    pub quota_units_text: String,
    pub visible: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_scheduled_allowlist_digest_view(item: &ScheduledAllowlistDigest) -> ScheduledAllowlistDigestView {
    ScheduledAllowlistDigestView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        audience: text::truncate(&item.audience, 80),
        quantity_text: item.quantity.to_string(),
        created_at: item.created_at.to_string(),
        sequence: item.sequence.to_string(),
        locale: item.locale.clone(),
        quota_units_text: item.quota_units.to_string(),
        visible: format!("{}", item.visible),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_allowlist_digest_views(items: &[ScheduledAllowlistDigest]) -> Vec<ScheduledAllowlistDigestView> {
    let mut views: Vec<ScheduledAllowlistDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_allowlist_digest_view(item));
    }
    views
}

pub fn sum_scheduled_allowlist_digest_view(view: &ScheduledAllowlistDigestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_created_at as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_allowlist_digest("code-764", "label-216");
        let view = to_scheduled_allowlist_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_allowlist_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
