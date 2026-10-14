use super::*;

use acme_kit::text;

/// How screens and exports show a primary escalation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticeView {
    pub code: String,
    pub name: String,
    pub port_text: String,
    pub label_text: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_primary_escalation_notice_view(item: &PrimaryEscalationNotice) -> PrimaryEscalationNoticeView {
    PrimaryEscalationNoticeView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        port_text: item.port.to_string(),
        label_text: text::truncate(&item.label, 64),
        scope_text: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_escalation_notice_views(items: &[PrimaryEscalationNotice]) -> Vec<PrimaryEscalationNoticeView> {
    let mut views: Vec<PrimaryEscalationNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_escalation_notice_view(item));
    }
    views
}

pub fn sum_primary_escalation_notice_view(view: &PrimaryEscalationNoticeView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_escalation_notice("code-351", "name-897");
        let view = to_primary_escalation_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_escalation_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
