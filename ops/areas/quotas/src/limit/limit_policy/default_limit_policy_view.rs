use super::*;

use acme_kit::text;

/// How screens and exports show a default limit policy.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyView {
    pub code: String,
    pub label: String,
    pub region: String,
    pub channel: String,
    pub pinned_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_default_limit_policy_view(item: &DefaultLimitPolicy) -> DefaultLimitPolicyView {
    DefaultLimitPolicyView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        region: item.region.clone(),
        channel: item.channel.clone(),
        pinned_text: format!("{}", item.pinned),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_limit_policy_views(items: &[DefaultLimitPolicy]) -> Vec<DefaultLimitPolicyView> {
    let mut views: Vec<DefaultLimitPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_limit_policy_view(item));
    }
    views
}

pub fn sum_default_limit_policy_view(view: &DefaultLimitPolicyView) -> Result<f64, String> {
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
        let item = new_default_limit_policy("code-565", "label-11");
        let view = to_default_limit_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_limit_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
