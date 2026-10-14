use super::*;

use acme_kit::text;

/// How screens and exports show a tiered page snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageSnapshotView {
    pub code: String,
    pub title: String,
    pub ends_at: String,
    pub rate: String,
    pub host_text: String,
    pub rotation_days: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_tiered_page_snapshot_view(item: &TieredPageSnapshot) -> TieredPageSnapshotView {
    TieredPageSnapshotView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        rate: format!("{:.2}", item.rate),
        host_text: item.host.clone(),
        rotation_days: item.rotation_days.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_page_snapshot_views(items: &[TieredPageSnapshot]) -> Vec<TieredPageSnapshotView> {
    let mut views: Vec<TieredPageSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_page_snapshot_view(item));
    }
    views
}

pub fn sum_tiered_page_snapshot_view(view: &TieredPageSnapshotView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_rate + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_page_snapshot("code-400", "title-393");
        let view = to_tiered_page_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_page_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
