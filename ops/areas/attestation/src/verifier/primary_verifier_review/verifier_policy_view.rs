use super::*;

use acme_kit::text;

/// How screens and exports show a verifier policy.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPolicyView {
    pub code_text: String,
    pub title_text: String,
    pub capacity: String,
    pub channel: String,
    pub cost: String,
    pub burst: String,
    pub priority_text: String,
    pub weight: String,
    pub checksum: String,
    pub threshold: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_verifier_policy_view(item: &VerifierPolicy) -> VerifierPolicyView {
    VerifierPolicyView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        capacity: item.capacity.to_string(),
        channel: item.channel.clone(),
        cost: format!("{:.2}", item.cost),
        burst: item.burst.to_string(),
        priority_text: item.priority.to_string(),
        weight: format!("{:.2}", item.weight),
        checksum: text::truncate(&item.checksum, 64),
        threshold: format!("{:.2}", item.threshold),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_policy_views(items: &[VerifierPolicy]) -> Vec<VerifierPolicyView> {
    let mut views: Vec<VerifierPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_policy_view(item));
    }
    views
}

pub fn sum_verifier_policy_view(view: &VerifierPolicyView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_cost + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_policy("code-292", "title-53");
        let view = to_verifier_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
