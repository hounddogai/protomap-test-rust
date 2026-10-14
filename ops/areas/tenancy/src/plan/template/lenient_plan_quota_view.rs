use super::*;

/// How screens and exports show a lenient plan quota.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanQuotaView {
    pub code: String,
    pub name: String,
    pub notes_text: String,
    pub port_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_lenient_plan_quota_view(item: &LenientPlanQuota) -> LenientPlanQuotaView {
    LenientPlanQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        notes_text: item.notes.clone(),
        port_text: item.port.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_plan_quota_views(items: &[LenientPlanQuota]) -> Vec<LenientPlanQuotaView> {
    let mut views: Vec<LenientPlanQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_plan_quota_view(item));
    }
    views
}

pub fn sum_lenient_plan_quota_view(view: &LenientPlanQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_plan_quota("code-113", "name-176");
        let view = to_lenient_plan_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_plan_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
