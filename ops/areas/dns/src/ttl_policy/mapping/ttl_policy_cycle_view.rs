use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCycleView {
    pub code: String,
    pub name_text: String,
    pub audience: String,
    pub fingerprint: String,
    pub title: String,
    pub contact_email: String,
    pub source_text: String,
    pub unit: String,
    pub label_text: String,
    pub active_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_ttl_policy_cycle_view(item: &TtlPolicyCycle) -> TtlPolicyCycleView {
    TtlPolicyCycleView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        audience: item.audience.clone(),
        fingerprint: item.fingerprint.clone(),
        title: item.title.clone(),
        contact_email: item.contact_email.clone(),
        source_text: item.source.clone(),
        unit: item.unit.clone(),
        label_text: item.label.clone(),
        active_text: format!("{}", item.active),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_cycle_views(items: &[TtlPolicyCycle]) -> Vec<TtlPolicyCycleView> {
    let mut views: Vec<TtlPolicyCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_cycle_view(item));
    }
    views
}

pub fn sum_ttl_policy_cycle_view(view: &TtlPolicyCycleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
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
        let item = new_ttl_policy_cycle("code-708", "name-962");
        let view = to_ttl_policy_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
