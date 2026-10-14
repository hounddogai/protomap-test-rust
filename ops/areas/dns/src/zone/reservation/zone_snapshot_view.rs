use super::*;

use acme_kit::text;

/// How screens and exports show a zone snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshotView {
    pub code: String,
    pub name: String,
    pub grace_days: String,
    pub expires_at: String,
    pub quantity: String,
    pub attempts_text: String,
    pub retries: String,
    pub weight: String,
    pub namespace: String,
    pub ends_at_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_zone_snapshot_view(item: &ZoneSnapshot) -> ZoneSnapshotView {
    ZoneSnapshotView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        grace_days: item.grace_days.to_string(),
        expires_at: item.expires_at.to_string(),
        quantity: item.quantity.to_string(),
        attempts_text: item.attempts.to_string(),
        retries: item.retries.to_string(),
        weight: format!("{:.2}", item.weight),
        namespace: item.namespace.clone(),
        ends_at_text: item.ends_at.to_string(),
        phone: text::truncate(&item.phone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_snapshot_views(items: &[ZoneSnapshot]) -> Vec<ZoneSnapshotView> {
    let mut views: Vec<ZoneSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_snapshot_view(item));
    }
    views
}

pub fn sum_zone_snapshot_view(view: &ZoneSnapshotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_expires_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_snapshot("code-677", "name-941");
        let view = to_zone_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
