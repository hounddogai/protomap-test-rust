use super::*;

use acme_kit::text;

/// How screens and exports show a sink snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSnapshotView {
    pub code: String,
    pub label: String,
    pub window_days_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_sink_snapshot_view(item: &SinkSnapshot) -> SinkSnapshotView {
    SinkSnapshotView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        window_days_text: item.window_days.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_snapshot_views(items: &[SinkSnapshot]) -> Vec<SinkSnapshotView> {
    let mut views: Vec<SinkSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_snapshot_view(item));
    }
    views
}

pub fn sum_sink_snapshot_view(view: &SinkSnapshotView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_snapshot("code-69", "label-86");
        let view = to_sink_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
