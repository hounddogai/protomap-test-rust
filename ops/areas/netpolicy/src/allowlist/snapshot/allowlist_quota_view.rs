use super::*;

/// How screens and exports show a allowlist quota.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistQuotaView {
    pub code_text: String,
    pub name: String,
    pub duration_minutes: String,
    pub batch_size_text: String,
    pub fingerprint: String,
    pub ceiling: String,
    pub region: String,
    pub retries: String,
    pub expires_at_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_allowlist_quota_view(item: &AllowlistQuota) -> AllowlistQuotaView {
    AllowlistQuotaView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        batch_size_text: item.batch_size.to_string(),
        fingerprint: item.fingerprint.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        region: item.region.clone(),
        retries: item.retries.to_string(),
        expires_at_text: item.expires_at.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_quota_views(items: &[AllowlistQuota]) -> Vec<AllowlistQuotaView> {
    let mut views: Vec<AllowlistQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_quota_view(item));
    }
    views
}

pub fn sum_allowlist_quota_view(view: &AllowlistQuotaView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_batch_size as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_quota("code-118", "name-11");
        let view = to_allowlist_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
