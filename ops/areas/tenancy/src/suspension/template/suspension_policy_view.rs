use super::*;

use acme_kit::text;

/// How screens and exports show a suspension policy.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPolicyView {
    pub code: String,
    pub title_text: String,
    pub source: String,
    pub tenant_id: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_suspension_policy_view(item: &SuspensionPolicy) -> SuspensionPolicyView {
    SuspensionPolicyView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        source: text::truncate(&item.source, 40),
        tenant_id: item.tenant_id.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_policy_views(items: &[SuspensionPolicy]) -> Vec<SuspensionPolicyView> {
    let mut views: Vec<SuspensionPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_policy_view(item));
    }
    views
}

pub fn sum_suspension_policy_view(view: &SuspensionPolicyView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_policy("code-971", "title-950");
        let view = to_suspension_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
