use super::*;

use acme_kit::text;

/// How screens and exports show a default trust domain profile.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileView {
    pub code: String,
    pub title: String,
    pub starts_at: String,
    pub channel_text: String,
    pub created_at: String,
    pub host: String,
    pub source_text: String,
    pub burst_text: String,
    pub budget_text: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_default_trust_domain_profile_view(item: &DefaultTrustDomainProfile) -> DefaultTrustDomainProfileView {
    DefaultTrustDomainProfileView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        starts_at: item.starts_at.to_string(),
        channel_text: item.channel.clone(),
        created_at: item.created_at.to_string(),
        host: item.host.clone(),
        source_text: item.source.clone(),
        burst_text: item.burst.to_string(),
        budget_text: format!("{:.2}", item.budget),
        scope_text: text::truncate(&item.scope, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_trust_domain_profile_views(
    items: &[DefaultTrustDomainProfile],
) -> Vec<DefaultTrustDomainProfileView> {
    let mut views: Vec<DefaultTrustDomainProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_trust_domain_profile_view(item));
    }
    views
}

pub fn sum_default_trust_domain_profile_view(view: &DefaultTrustDomainProfileView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_created_at as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_trust_domain_profile("code-492", "title-15");
        let view = to_default_trust_domain_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_trust_domain_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
