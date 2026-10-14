use super::*;

/// How screens and exports show a failover manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverManifestView {
    pub code_text: String,
    pub title: String,
    pub host_text: String,
    pub contact_email_text: String,
    pub principal: String,
    pub port: String,
    pub burst_text: String,
    pub ttl_seconds_text: String,
    pub sequence: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_failover_manifest_view(item: &FailoverManifest) -> FailoverManifestView {
    FailoverManifestView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        host_text: item.host.clone(),
        contact_email_text: item.contact_email.clone(),
        principal: item.principal.clone(),
        port: item.port.to_string(),
        burst_text: item.burst.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        sequence: item.sequence.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_manifest_views(items: &[FailoverManifest]) -> Vec<FailoverManifestView> {
    let mut views: Vec<FailoverManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_manifest_view(item));
    }
    views
}

pub fn sum_failover_manifest_view(view: &FailoverManifestView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_port as f64 + (parsed_burst as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_manifest("code-385", "title-793");
        let view = to_failover_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
