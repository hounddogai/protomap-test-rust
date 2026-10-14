use super::*;

use acme_kit::text;

/// How screens and exports show a strict rollback policy.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRollbackPolicyView {
    pub code: String,
    pub name_text: String,
    pub tenant_id_text: String,
    pub description_text: String,
    pub summary_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_strict_rollback_policy_view(item: &StrictRollbackPolicy) -> StrictRollbackPolicyView {
    StrictRollbackPolicyView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        tenant_id_text: item.tenant_id.clone(),
        description_text: item.description.clone(),
        summary_text: item.summary.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_rollback_policy_views(items: &[StrictRollbackPolicy]) -> Vec<StrictRollbackPolicyView> {
    let mut views: Vec<StrictRollbackPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_rollback_policy_view(item));
    }
    views
}

pub fn sum_strict_rollback_policy_view(view: &StrictRollbackPolicyView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_rollback_policy("code-595", "name-849");
        let view = to_strict_rollback_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_rollback_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
