use super::*;

use acme_kit::text;

/// How screens and exports show a legacy probe group.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupView {
    pub code: String,
    pub name: String,
    pub port: String,
    pub retries_text: String,
    pub principal_text: String,
    pub segment: String,
    pub source: String,
    pub grace_days: String,
    pub cipher: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_legacy_probe_group_view(item: &LegacyProbeGroup) -> LegacyProbeGroupView {
    LegacyProbeGroupView {
        code: item.code.clone(),
        name: item.name.clone(),
        port: item.port.to_string(),
        retries_text: item.retries.to_string(),
        principal_text: item.principal.clone(),
        segment: item.segment.clone(),
        source: item.source.clone(),
        grace_days: item.grace_days.to_string(),
        cipher: text::truncate(&item.cipher, 120),
        phone: text::truncate(&item.phone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_probe_group_views(items: &[LegacyProbeGroup]) -> Vec<LegacyProbeGroupView> {
    let mut views: Vec<LegacyProbeGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_probe_group_view(item));
    }
    views
}

pub fn sum_legacy_probe_group_view(view: &LegacyProbeGroupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_port as f64 + (parsed_retries as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_probe_group("code-704", "name-215");
        let view = to_legacy_probe_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_probe_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
