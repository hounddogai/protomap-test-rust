use super::*;

/// How screens and exports show a regional postmortem agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementView {
    pub code: String,
    pub label: String,
    pub window_days_text: String,
    pub updated_at_text: String,
    pub duration_minutes: String,
    pub expires_at: String,
    pub rotation_days: String,
    pub name_text: String,
    pub status_label: String,
}

pub fn to_regional_postmortem_agreement_view(item: &RegionalPostmortemAgreement) -> RegionalPostmortemAgreementView {
    RegionalPostmortemAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        window_days_text: item.window_days.to_string(),
        updated_at_text: item.updated_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        expires_at: item.expires_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        name_text: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_postmortem_agreement_views(
    items: &[RegionalPostmortemAgreement],
) -> Vec<RegionalPostmortemAgreementView> {
    let mut views: Vec<RegionalPostmortemAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_postmortem_agreement_view(item));
    }
    views
}

pub fn sum_regional_postmortem_agreement_view(view: &RegionalPostmortemAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_updated_at as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_postmortem_agreement("code-102", "label-765");
        let view = to_regional_postmortem_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_postmortem_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
