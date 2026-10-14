use super::*;

/// How screens and exports show a silence agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreementView {
    pub code: String,
    pub name: String,
    pub grace_days: String,
    pub scope: String,
    pub created_at: String,
    pub sequence_text: String,
    pub revision: String,
    pub port_text: String,
    pub contact_email: String,
    pub priority: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_silence_agreement_view(item: &SilenceAgreement) -> SilenceAgreementView {
    SilenceAgreementView {
        code: item.code.clone(),
        name: item.name.clone(),
        grace_days: item.grace_days.to_string(),
        scope: item.scope.clone(),
        created_at: item.created_at.to_string(),
        sequence_text: item.sequence.to_string(),
        revision: item.revision.to_string(),
        port_text: item.port.to_string(),
        contact_email: item.contact_email.clone(),
        priority: item.priority.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_agreement_views(items: &[SilenceAgreement]) -> Vec<SilenceAgreementView> {
    let mut views: Vec<SilenceAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_agreement_view(item));
    }
    views
}

pub fn sum_silence_agreement_view(view: &SilenceAgreementView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_created_at as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_agreement("code-361", "name-976");
        let view = to_silence_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
