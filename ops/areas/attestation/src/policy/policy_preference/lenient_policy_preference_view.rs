use super::*;

use acme_kit::text;

/// How screens and exports show a lenient policy preference.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyPreferenceView {
    pub code: String,
    pub label: String,
    pub region: String,
    pub starts_at: String,
    pub ends_at: String,
    pub checksum_text: String,
    pub channel: String,
    pub ceiling_text: String,
    pub locale: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_lenient_policy_preference_view(item: &LenientPolicyPreference) -> LenientPolicyPreferenceView {
    LenientPolicyPreferenceView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        region: item.region.clone(),
        starts_at: item.starts_at.to_string(),
        ends_at: item.ends_at.to_string(),
        checksum_text: item.checksum.clone(),
        channel: text::truncate(&item.channel, 80),
        ceiling_text: format!("{:.2}", item.ceiling),
        locale: item.locale.clone(),
        phone: text::truncate(&item.phone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_policy_preference_views(items: &[LenientPolicyPreference]) -> Vec<LenientPolicyPreferenceView> {
    let mut views: Vec<LenientPolicyPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_policy_preference_view(item));
    }
    views
}

pub fn sum_lenient_policy_preference_view(view: &LenientPolicyPreferenceView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_ends_at as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_policy_preference("code-123", "label-71");
        let view = to_lenient_policy_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_policy_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
