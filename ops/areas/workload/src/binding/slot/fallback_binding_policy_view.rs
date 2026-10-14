use super::*;

use acme_kit::text;

/// How screens and exports show a fallback binding policy.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingPolicyView {
    pub code: String,
    pub name: String,
    pub active: String,
    pub visible: String,
    pub discount: String,
    pub audience: String,
    pub revision: String,
    pub score: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_fallback_binding_policy_view(item: &FallbackBindingPolicy) -> FallbackBindingPolicyView {
    FallbackBindingPolicyView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        active: format!("{}", item.active),
        visible: format!("{}", item.visible),
        discount: format!("{:.2}", item.discount),
        audience: item.audience.clone(),
        revision: item.revision.to_string(),
        score: format!("{:.2}", item.score),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_binding_policy_views(items: &[FallbackBindingPolicy]) -> Vec<FallbackBindingPolicyView> {
    let mut views: Vec<FallbackBindingPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_binding_policy_view(item));
    }
    views
}

pub fn sum_fallback_binding_policy_view(view: &FallbackBindingPolicyView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_discount + (parsed_revision as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_binding_policy("code-528", "name-941");
        let view = to_fallback_binding_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_binding_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
