use super::*;

/// How screens and exports show a default limit audit.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitAuditView {
    pub code: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub category: String,
    pub floor_value: String,
    pub sequence: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_default_limit_audit_view(item: &DefaultLimitAudit) -> DefaultLimitAuditView {
    DefaultLimitAuditView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        category: item.category.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        sequence: item.sequence.to_string(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_limit_audit_views(items: &[DefaultLimitAudit]) -> Vec<DefaultLimitAuditView> {
    let mut views: Vec<DefaultLimitAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_limit_audit_view(item));
    }
    views
}

pub fn sum_default_limit_audit_view(view: &DefaultLimitAuditView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_floor_value + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_limit_audit("code-759", "label-784");
        let view = to_default_limit_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_limit_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
