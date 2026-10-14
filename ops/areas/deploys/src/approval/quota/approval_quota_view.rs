use super::*;

use acme_kit::text;

/// How screens and exports show a approval quota.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaView {
    pub code: String,
    pub name: String,
    pub port_text: String,
    pub fingerprint: String,
    pub ceiling: String,
    pub channel: String,
    pub issuer_text: String,
    pub retries: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_approval_quota_view(item: &ApprovalQuota) -> ApprovalQuotaView {
    ApprovalQuotaView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        port_text: item.port.to_string(),
        fingerprint: item.fingerprint.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        channel: item.channel.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        retries: item.retries.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_quota_views(items: &[ApprovalQuota]) -> Vec<ApprovalQuotaView> {
    let mut views: Vec<ApprovalQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_quota_view(item));
    }
    views
}

pub fn sum_approval_quota_view(view: &ApprovalQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_port as f64 + parsed_ceiling + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_quota("code-586", "name-267");
        let view = to_approval_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
