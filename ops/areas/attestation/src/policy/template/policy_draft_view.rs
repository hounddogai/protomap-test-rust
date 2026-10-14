use super::*;

use acme_kit::text;

/// How screens and exports show a policy draft.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftView {
    pub code_text: String,
    pub name_text: String,
    pub port: String,
    pub issuer: String,
    pub ceiling: String,
    pub timezone: String,
    pub key_id: String,
    pub locale_text: String,
    pub maximum: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_policy_draft_view(item: &PolicyDraft) -> PolicyDraftView {
    PolicyDraftView {
        code_text: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 120),
        port: item.port.to_string(),
        issuer: item.issuer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        timezone: text::truncate(&item.timezone, 80),
        key_id: item.key_id.clone(),
        locale_text: text::truncate(&item.locale, 120),
        maximum: format!("{:.2}", item.maximum),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_draft_views(items: &[PolicyDraft]) -> Vec<PolicyDraftView> {
    let mut views: Vec<PolicyDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_draft_view(item));
    }
    views
}

pub fn sum_policy_draft_view(view: &PolicyDraftView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_port as f64 + parsed_ceiling + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_draft("code-530", "name-355");
        let view = to_policy_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
