use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy tier.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierView {
    pub code_text: String,
    pub name: String,
    pub segment_text: String,
    pub title: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_batched_policy_tier_view(item: &BatchedPolicyTier) -> BatchedPolicyTierView {
    BatchedPolicyTierView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        segment_text: text::truncate(&item.segment, 40),
        title: item.title.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_tier_views(items: &[BatchedPolicyTier]) -> Vec<BatchedPolicyTierView> {
    let mut views: Vec<BatchedPolicyTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_tier_view(item));
    }
    views
}

pub fn sum_batched_policy_tier_view(view: &BatchedPolicyTierView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_policy_tier("code-788", "name-939");
        let view = to_batched_policy_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_policy_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
