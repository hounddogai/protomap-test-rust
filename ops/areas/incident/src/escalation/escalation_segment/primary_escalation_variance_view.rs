use super::*;

use acme_kit::text;

/// How screens and exports show a primary escalation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationVarianceView {
    pub code: String,
    pub title: String,
    pub port: String,
    pub grace_days_text: String,
    pub unit: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_primary_escalation_variance_view(item: &PrimaryEscalationVariance) -> PrimaryEscalationVarianceView {
    PrimaryEscalationVarianceView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        port: item.port.to_string(),
        grace_days_text: item.grace_days.to_string(),
        unit: text::truncate(&item.unit, 64),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_escalation_variance_views(items: &[PrimaryEscalationVariance]) -> Vec<PrimaryEscalationVarianceView> {
    let mut views: Vec<PrimaryEscalationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_escalation_variance_view(item));
    }
    views
}

pub fn sum_primary_escalation_variance_view(view: &PrimaryEscalationVarianceView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
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
        let item = new_primary_escalation_variance("code-23", "title-640");
        let view = to_primary_escalation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_escalation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
