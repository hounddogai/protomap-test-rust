use super::*;

use acme_kit::text;

/// How screens and exports show a rolling peering audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditView {
    pub code: String,
    pub name: String,
    pub archived_text: String,
    pub channel: String,
    pub rotation_days_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_rolling_peering_audit_view(item: &RollingPeeringAudit) -> RollingPeeringAuditView {
    RollingPeeringAuditView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        archived_text: format!("{}", item.archived),
        channel: text::truncate(&item.channel, 120),
        rotation_days_text: item.rotation_days.to_string(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_peering_audit_views(items: &[RollingPeeringAudit]) -> Vec<RollingPeeringAuditView> {
    let mut views: Vec<RollingPeeringAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_peering_audit_view(item));
    }
    views
}

pub fn sum_rolling_peering_audit_view(view: &RollingPeeringAuditView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_peering_audit("code-148", "name-599");
        let view = to_rolling_peering_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_peering_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
