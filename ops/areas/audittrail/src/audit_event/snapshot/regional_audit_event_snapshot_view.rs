use super::*;

/// How screens and exports show a regional audit event snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshotView {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub revision: String,
    pub rate: String,
    pub visible_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_regional_audit_event_snapshot_view(item: &RegionalAuditEventSnapshot) -> RegionalAuditEventSnapshotView {
    RegionalAuditEventSnapshotView {
        code: item.code.clone(),
        title: item.title.clone(),
        contact_email: item.contact_email.clone(),
        revision: item.revision.to_string(),
        rate: format!("{:.2}", item.rate),
        visible_text: format!("{}", item.visible),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_audit_event_snapshot_views(
    items: &[RegionalAuditEventSnapshot],
) -> Vec<RegionalAuditEventSnapshotView> {
    let mut views: Vec<RegionalAuditEventSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_audit_event_snapshot_view(item));
    }
    views
}

pub fn sum_regional_audit_event_snapshot_view(view: &RegionalAuditEventSnapshotView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_revision as f64 + parsed_rate + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_audit_event_snapshot("code-776", "title-134");
        let view = to_regional_audit_event_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_audit_event_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
