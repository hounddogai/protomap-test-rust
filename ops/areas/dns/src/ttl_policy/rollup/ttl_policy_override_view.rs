use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy override.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyOverrideView {
    pub code: String,
    pub title_text: String,
    pub minimum: String,
    pub sequence: String,
    pub country: String,
    pub score_text: String,
    pub reviewer: String,
    pub threshold_text: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_ttl_policy_override_view(item: &TtlPolicyOverride) -> TtlPolicyOverrideView {
    TtlPolicyOverrideView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        sequence: item.sequence.to_string(),
        country: item.country.clone(),
        score_text: format!("{:.2}", item.score),
        reviewer: item.reviewer.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_override_views(items: &[TtlPolicyOverride]) -> Vec<TtlPolicyOverrideView> {
    let mut views: Vec<TtlPolicyOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_override_view(item));
    }
    views
}

pub fn sum_ttl_policy_override_view(view: &TtlPolicyOverrideView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_minimum + (parsed_sequence as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_override("code-341", "title-585");
        let view = to_ttl_policy_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
