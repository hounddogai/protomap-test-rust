use super::*;

/// How screens and exports show a lenient delegation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationPolicyView {
    pub code_text: String,
    pub title_text: String,
    pub country: String,
    pub capacity: String,
    pub channel: String,
    pub ttl_seconds: String,
    pub attempts_text: String,
    pub quantity: String,
    pub duration_minutes: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_lenient_delegation_policy_view(item: &LenientDelegationPolicy) -> LenientDelegationPolicyView {
    LenientDelegationPolicyView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        country: item.country.clone(),
        capacity: item.capacity.to_string(),
        channel: item.channel.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        attempts_text: item.attempts.to_string(),
        quantity: item.quantity.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_delegation_policy_views(items: &[LenientDelegationPolicy]) -> Vec<LenientDelegationPolicyView> {
    let mut views: Vec<LenientDelegationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_delegation_policy_view(item));
    }
    views
}

pub fn sum_lenient_delegation_policy_view(view: &LenientDelegationPolicyView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_ttl_seconds as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_delegation_policy("code-394", "title-971");
        let view = to_lenient_delegation_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_delegation_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
