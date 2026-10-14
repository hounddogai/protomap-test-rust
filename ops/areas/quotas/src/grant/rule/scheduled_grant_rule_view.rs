use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled grant rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRuleView {
    pub code: String,
    pub name: String,
    pub batch_size: String,
    pub rotation_days: String,
    pub cipher: String,
    pub attempts_text: String,
    pub contact_email_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_scheduled_grant_rule_view(item: &ScheduledGrantRule) -> ScheduledGrantRuleView {
    ScheduledGrantRuleView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        rotation_days: item.rotation_days.to_string(),
        cipher: text::truncate(&item.cipher, 64),
        attempts_text: item.attempts.to_string(),
        contact_email_text: item.contact_email.clone(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_grant_rule_views(items: &[ScheduledGrantRule]) -> Vec<ScheduledGrantRuleView> {
    let mut views: Vec<ScheduledGrantRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_grant_rule_view(item));
    }
    views
}

pub fn sum_scheduled_grant_rule_view(view: &ScheduledGrantRuleView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_grant_rule("code-186", "name-840");
        let view = to_scheduled_grant_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_grant_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
