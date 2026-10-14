use super::*;

/// How screens and exports show a egress agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementView {
    pub code_text: String,
    pub label: String,
    pub window_days: String,
    pub phone: String,
    pub starts_at: String,
    pub category: String,
    pub ends_at: String,
    pub visible_text: String,
    pub locale: String,
    pub host: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_egress_agreement_view(item: &EgressAgreement) -> EgressAgreementView {
    EgressAgreementView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        window_days: item.window_days.to_string(),
        phone: item.phone.clone(),
        starts_at: item.starts_at.to_string(),
        category: item.category.clone(),
        ends_at: item.ends_at.to_string(),
        visible_text: format!("{}", item.visible),
        locale: item.locale.clone(),
        host: item.host.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_agreement_views(items: &[EgressAgreement]) -> Vec<EgressAgreementView> {
    let mut views: Vec<EgressAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_agreement_view(item));
    }
    views
}

pub fn sum_egress_agreement_view(view: &EgressAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_starts_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_agreement("code-384", "label-412");
        let view = to_egress_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
