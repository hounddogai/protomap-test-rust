use super::*;

/// How screens and exports show a primary failover snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshotView {
    pub code_text: String,
    pub name: String,
    pub grace_days_text: String,
    pub minimum: String,
    pub approved: String,
    pub segment: String,
    pub updated_at: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_primary_failover_snapshot_view(item: &PrimaryFailoverSnapshot) -> PrimaryFailoverSnapshotView {
    PrimaryFailoverSnapshotView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        grace_days_text: item.grace_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        approved: format!("{}", item.approved),
        segment: item.segment.clone(),
        updated_at: item.updated_at.to_string(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_failover_snapshot_views(items: &[PrimaryFailoverSnapshot]) -> Vec<PrimaryFailoverSnapshotView> {
    let mut views: Vec<PrimaryFailoverSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_failover_snapshot_view(item));
    }
    views
}

pub fn sum_primary_failover_snapshot_view(view: &PrimaryFailoverSnapshotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_minimum + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_failover_snapshot("code-673", "name-550");
        let view = to_primary_failover_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_failover_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
