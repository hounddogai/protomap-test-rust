use super::*;

use acme_kit::text;

/// How screens and exports show a lenient redaction policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionPolicyView {
    pub code: String,
    pub name_text: String,
    pub retries: String,
    pub created_at: String,
    pub archived: String,
    pub pinned: String,
    pub ends_at: String,
    pub description: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_lenient_redaction_policy_view(item: &LenientRedactionPolicy) -> LenientRedactionPolicyView {
    LenientRedactionPolicyView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        retries: item.retries.to_string(),
        created_at: item.created_at.to_string(),
        archived: format!("{}", item.archived),
        pinned: format!("{}", item.pinned),
        ends_at: item.ends_at.to_string(),
        description: item.description.clone(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_redaction_policy_views(items: &[LenientRedactionPolicy]) -> Vec<LenientRedactionPolicyView> {
    let mut views: Vec<LenientRedactionPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_redaction_policy_view(item));
    }
    views
}

pub fn sum_lenient_redaction_policy_view(view: &LenientRedactionPolicyView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_created_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_redaction_policy("code-878", "name-653");
        let view = to_lenient_redaction_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_redaction_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
