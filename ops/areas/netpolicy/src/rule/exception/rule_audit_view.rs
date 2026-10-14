use super::*;

/// How screens and exports show a rule audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAuditView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub sequence_text: String,
    pub minimum: String,
    pub retries_text: String,
    pub title: String,
    pub weight_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_rule_audit_view(item: &RuleAudit) -> RuleAuditView {
    RuleAuditView {
        code: item.code.clone(),
        name: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        sequence_text: item.sequence.to_string(),
        minimum: format!("{:.2}", item.minimum),
        retries_text: item.retries.to_string(),
        title: item.title.clone(),
        weight_text: format!("{:.2}", item.weight),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_audit_views(items: &[RuleAudit]) -> Vec<RuleAuditView> {
    let mut views: Vec<RuleAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_audit_view(item));
    }
    views
}

pub fn sum_rule_audit_view(view: &RuleAuditView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_minimum + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_audit("code-580", "name-853");
        let view = to_rule_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
