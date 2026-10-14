use super::*;

/// How screens and exports show a manual peering snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringSnapshotView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub grace_days: String,
    pub pinned_text: String,
    pub channel: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_manual_peering_snapshot_view(item: &ManualPeeringSnapshot) -> ManualPeeringSnapshotView {
    ManualPeeringSnapshotView {
        code: item.code.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        grace_days: item.grace_days.to_string(),
        pinned_text: format!("{}", item.pinned),
        channel: item.channel.clone(),
        contact_email_text: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_peering_snapshot_views(items: &[ManualPeeringSnapshot]) -> Vec<ManualPeeringSnapshotView> {
    let mut views: Vec<ManualPeeringSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_peering_snapshot_view(item));
    }
    views
}

pub fn sum_manual_peering_snapshot_view(view: &ManualPeeringSnapshotView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_port as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_peering_snapshot("code-153", "label-914");
        let view = to_manual_peering_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_peering_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
