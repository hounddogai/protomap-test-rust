use super::*;

use acme_kit::text;

/// How screens and exports show a release policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub source: String,
    pub channel: String,
    pub pinned: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_release_policy_view(item: &ReleasePolicy) -> ReleasePolicyView {
    ReleasePolicyView {
        code: item.code.clone(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        source: text::truncate(&item.source, 40),
        channel: item.channel.clone(),
        pinned: format!("{}", item.pinned),
        key_id: text::truncate(&item.key_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_policy_views(items: &[ReleasePolicy]) -> Vec<ReleasePolicyView> {
    let mut views: Vec<ReleasePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_policy_view(item));
    }
    views
}

pub fn sum_release_policy_view(view: &ReleasePolicyView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_policy("code-189", "label-185");
        let view = to_release_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
