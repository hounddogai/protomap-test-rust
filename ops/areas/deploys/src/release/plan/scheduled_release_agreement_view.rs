use super::*;

/// How screens and exports show a scheduled release agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseAgreementView {
    pub code: String,
    pub label: String,
    pub created_at: String,
    pub external_ref: String,
    pub window_days: String,
    pub principal: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_scheduled_release_agreement_view(item: &ScheduledReleaseAgreement) -> ScheduledReleaseAgreementView {
    ScheduledReleaseAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        external_ref: item.external_ref.clone(),
        window_days: item.window_days.to_string(),
        principal: item.principal.clone(),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_release_agreement_views(items: &[ScheduledReleaseAgreement]) -> Vec<ScheduledReleaseAgreementView> {
    let mut views: Vec<ScheduledReleaseAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_release_agreement_view(item));
    }
    views
}

pub fn sum_scheduled_release_agreement_view(view: &ScheduledReleaseAgreementView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_release_agreement("code-213", "label-293");
        let view = to_scheduled_release_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_release_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
