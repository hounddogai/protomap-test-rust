use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled retention digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigestView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub notes_text: String,
    pub description: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_scheduled_retention_digest_view(item: &ScheduledRetentionDigest) -> ScheduledRetentionDigestView {
    ScheduledRetentionDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        owner_id: item.owner_id.clone(),
        notes_text: item.notes.clone(),
        description: text::truncate(&item.description, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_retention_digest_views(items: &[ScheduledRetentionDigest]) -> Vec<ScheduledRetentionDigestView> {
    let mut views: Vec<ScheduledRetentionDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_retention_digest_view(item));
    }
    views
}

pub fn sum_scheduled_retention_digest_view(view: &ScheduledRetentionDigestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_retention_digest("code-79", "title-581");
        let view = to_scheduled_retention_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_retention_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
