use super::*;

/// How screens and exports show a seasonal dashboard snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardSnapshotView {
    pub code: String,
    pub label_text: String,
    pub ends_at: String,
    pub sequence_text: String,
    pub maximum: String,
    pub pinned: String,
    pub cluster: String,
    pub cost: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_seasonal_dashboard_snapshot_view(item: &SeasonalDashboardSnapshot) -> SeasonalDashboardSnapshotView {
    SeasonalDashboardSnapshotView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        sequence_text: item.sequence.to_string(),
        maximum: format!("{:.2}", item.maximum),
        pinned: format!("{}", item.pinned),
        cluster: item.cluster.clone(),
        cost: format!("{:.2}", item.cost),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_dashboard_snapshot_views(items: &[SeasonalDashboardSnapshot]) -> Vec<SeasonalDashboardSnapshotView> {
    let mut views: Vec<SeasonalDashboardSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_dashboard_snapshot_view(item));
    }
    views
}

pub fn sum_seasonal_dashboard_snapshot_view(view: &SeasonalDashboardSnapshotView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_sequence as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_dashboard_snapshot("code-231", "label-564");
        let view = to_seasonal_dashboard_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_dashboard_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
