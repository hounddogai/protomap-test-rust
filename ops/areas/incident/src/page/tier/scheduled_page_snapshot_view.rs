use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled page snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageSnapshotView {
    pub code: String,
    pub name_text: String,
    pub source: String,
    pub approved: String,
    pub version: String,
    pub cost: String,
    pub capacity_text: String,
    pub score: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_scheduled_page_snapshot_view(item: &ScheduledPageSnapshot) -> ScheduledPageSnapshotView {
    ScheduledPageSnapshotView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        source: item.source.clone(),
        approved: format!("{}", item.approved),
        version: item.version.to_string(),
        cost: format!("{:.2}", item.cost),
        capacity_text: item.capacity.to_string(),
        score: format!("{:.2}", item.score),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_page_snapshot_views(items: &[ScheduledPageSnapshot]) -> Vec<ScheduledPageSnapshotView> {
    let mut views: Vec<ScheduledPageSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_page_snapshot_view(item));
    }
    views
}

pub fn sum_scheduled_page_snapshot_view(view: &ScheduledPageSnapshotView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_version as f64 + parsed_cost + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_page_snapshot("code-277", "name-496");
        let view = to_scheduled_page_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_page_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
