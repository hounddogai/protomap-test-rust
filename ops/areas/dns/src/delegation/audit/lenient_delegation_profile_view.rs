use super::*;

use acme_kit::text;

/// How screens and exports show a lenient delegation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfileView {
    pub code: String,
    pub title_text: String,
    pub cluster: String,
    pub priority: String,
    pub attempts: String,
    pub ceiling: String,
    pub fingerprint: String,
    pub budget_text: String,
    pub approved: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_lenient_delegation_profile_view(item: &LenientDelegationProfile) -> LenientDelegationProfileView {
    LenientDelegationProfileView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        cluster: text::truncate(&item.cluster, 120),
        priority: item.priority.to_string(),
        attempts: item.attempts.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        fingerprint: item.fingerprint.clone(),
        budget_text: format!("{:.2}", item.budget),
        approved: format!("{}", item.approved),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_delegation_profile_views(items: &[LenientDelegationProfile]) -> Vec<LenientDelegationProfileView> {
    let mut views: Vec<LenientDelegationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_delegation_profile_view(item));
    }
    views
}

pub fn sum_lenient_delegation_profile_view(view: &LenientDelegationProfileView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_attempts as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_delegation_profile("code-857", "title-375");
        let view = to_lenient_delegation_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_delegation_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
