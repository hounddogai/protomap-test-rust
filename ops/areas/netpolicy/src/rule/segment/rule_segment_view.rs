use super::*;

use acme_kit::text;

/// How screens and exports show a rule segment.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegmentView {
    pub code_text: String,
    pub title: String,
    pub label_text: String,
    pub priority: String,
    pub fingerprint: String,
    pub segment: String,
    pub timezone: String,
    pub grace_days: String,
    pub active: String,
    pub contact_email: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_rule_segment_view(item: &RuleSegment) -> RuleSegmentView {
    RuleSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        label_text: text::truncate(&item.label, 80),
        priority: item.priority.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        segment: item.segment.clone(),
        timezone: item.timezone.clone(),
        grace_days: item.grace_days.to_string(),
        active: format!("{}", item.active),
        contact_email: item.contact_email.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_segment_views(items: &[RuleSegment]) -> Vec<RuleSegmentView> {
    let mut views: Vec<RuleSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_segment_view(item));
    }
    views
}

pub fn sum_rule_segment_view(view: &RuleSegmentView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_segment("code-605", "title-88");
        let view = to_rule_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
