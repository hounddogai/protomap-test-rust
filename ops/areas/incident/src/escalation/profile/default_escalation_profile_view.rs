use super::*;

use acme_kit::text;

/// How screens and exports show a default escalation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationProfileView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub port: String,
    pub issuer: String,
    pub rate_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_default_escalation_profile_view(item: &DefaultEscalationProfile) -> DefaultEscalationProfileView {
    DefaultEscalationProfileView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        port: item.port.to_string(),
        issuer: item.issuer.clone(),
        rate_text: format!("{:.2}", item.rate),
        channel: text::truncate(&item.channel, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_escalation_profile_views(items: &[DefaultEscalationProfile]) -> Vec<DefaultEscalationProfileView> {
    let mut views: Vec<DefaultEscalationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_escalation_profile_view(item));
    }
    views
}

pub fn sum_default_escalation_profile_view(view: &DefaultEscalationProfileView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_port as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_escalation_profile("code-288", "title-401");
        let view = to_default_escalation_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_escalation_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
