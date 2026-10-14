use super::*;

/// How screens and exports show a slo snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSnapshotView {
    pub code: String,
    pub label_text: String,
    pub minimum_text: String,
    pub duration_minutes: String,
    pub fingerprint: String,
    pub audience: String,
    pub limit_text: String,
    pub approved_text: String,
    pub priority: String,
    pub timezone: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_slo_snapshot_view(item: &SloSnapshot) -> SloSnapshotView {
    SloSnapshotView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        duration_minutes: item.duration_minutes.to_string(),
        fingerprint: item.fingerprint.clone(),
        audience: item.audience.clone(),
        limit_text: item.limit.to_string(),
        approved_text: format!("{}", item.approved),
        priority: item.priority.to_string(),
        timezone: item.timezone.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_snapshot_views(items: &[SloSnapshot]) -> Vec<SloSnapshotView> {
    let mut views: Vec<SloSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_snapshot_view(item));
    }
    views
}

pub fn sum_slo_snapshot_view(view: &SloSnapshotView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_minimum + (parsed_duration_minutes as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_snapshot("code-205", "label-969");
        let view = to_slo_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
