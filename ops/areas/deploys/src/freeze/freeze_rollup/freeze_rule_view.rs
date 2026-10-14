use super::*;

use acme_kit::text;

/// How screens and exports show a freeze rule.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRuleView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub revision_text: String,
    pub tenant_id_text: String,
    pub pinned: String,
    pub grace_days: String,
    pub currency: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_freeze_rule_view(item: &FreezeRule) -> FreezeRuleView {
    FreezeRuleView {
        code: item.code.clone(),
        name: item.name.clone(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        revision_text: item.revision.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        currency: text::truncate(&item.currency, 64),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_rule_views(items: &[FreezeRule]) -> Vec<FreezeRuleView> {
    let mut views: Vec<FreezeRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_rule_view(item));
    }
    views
}

pub fn sum_freeze_rule_view(view: &FreezeRuleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_grace_days as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_rule("code-224", "name-576");
        let view = to_freeze_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
