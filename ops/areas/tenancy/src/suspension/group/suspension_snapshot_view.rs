use super::*;

use acme_kit::text;

/// How screens and exports show a suspension snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSnapshotView {
    pub code: String,
    pub name_text: String,
    pub ends_at: String,
    pub rate_text: String,
    pub description: String,
    pub rotation_days: String,
    pub sequence_text: String,
    pub priority_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_suspension_snapshot_view(item: &SuspensionSnapshot) -> SuspensionSnapshotView {
    SuspensionSnapshotView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 120),
        ends_at: item.ends_at.to_string(),
        rate_text: format!("{:.2}", item.rate),
        description: item.description.clone(),
        rotation_days: item.rotation_days.to_string(),
        sequence_text: item.sequence.to_string(),
        priority_text: item.priority.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_snapshot_views(items: &[SuspensionSnapshot]) -> Vec<SuspensionSnapshotView> {
    let mut views: Vec<SuspensionSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_snapshot_view(item));
    }
    views
}

pub fn sum_suspension_snapshot_view(view: &SuspensionSnapshotView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_rate = view
        .rate_text
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
        let item = new_suspension_snapshot("code-389", "name-467");
        let view = to_suspension_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
