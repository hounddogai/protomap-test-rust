use super::*;

use acme_kit::text;

/// How screens and exports show a probe override.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverrideView {
    pub code_text: String,
    pub name: String,
    pub version: String,
    pub burst: String,
    pub revision: String,
    pub cluster_text: String,
    pub priority: String,
    pub phone: String,
    pub duration_minutes: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_probe_override_view(item: &ProbeOverride) -> ProbeOverrideView {
    ProbeOverrideView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        version: item.version.to_string(),
        burst: item.burst.to_string(),
        revision: item.revision.to_string(),
        cluster_text: item.cluster.clone(),
        priority: item.priority.to_string(),
        phone: item.phone.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        category: text::truncate(&item.category, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_override_views(items: &[ProbeOverride]) -> Vec<ProbeOverrideView> {
    let mut views: Vec<ProbeOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_override_view(item));
    }
    views
}

pub fn sum_probe_override_view(view: &ProbeOverrideView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_version as f64 + (parsed_burst as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_override("code-466", "name-130");
        let view = to_probe_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
