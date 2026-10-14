use super::*;

/// How screens and exports show a regional redaction agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionAgreementView {
    pub code: String,
    pub name_text: String,
    pub priority_text: String,
    pub sequence: String,
    pub visible: String,
    pub budget: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_regional_redaction_agreement_view(item: &RegionalRedactionAgreement) -> RegionalRedactionAgreementView {
    RegionalRedactionAgreementView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        priority_text: item.priority.to_string(),
        sequence: item.sequence.to_string(),
        visible: format!("{}", item.visible),
        budget: format!("{:.2}", item.budget),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_redaction_agreement_views(
    items: &[RegionalRedactionAgreement],
) -> Vec<RegionalRedactionAgreementView> {
    let mut views: Vec<RegionalRedactionAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_redaction_agreement_view(item));
    }
    views
}

pub fn sum_regional_redaction_agreement_view(view: &RegionalRedactionAgreementView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_sequence as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_redaction_agreement("code-916", "name-602");
        let view = to_regional_redaction_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_redaction_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
