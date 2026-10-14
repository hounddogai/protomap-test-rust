use super::*;

use acme_kit::text;

/// How screens and exports show a verifier override.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideView {
    pub code_text: String,
    pub title: String,
    pub duration_minutes: String,
    pub cluster: String,
    pub port: String,
    pub retries: String,
    pub limit_text: String,
    pub namespace: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_verifier_override_view(item: &VerifierOverride) -> VerifierOverrideView {
    VerifierOverrideView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        cluster: text::truncate(&item.cluster, 120),
        port: item.port.to_string(),
        retries: item.retries.to_string(),
        limit_text: item.limit.to_string(),
        namespace: text::truncate(&item.namespace, 40),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_override_views(items: &[VerifierOverride]) -> Vec<VerifierOverrideView> {
    let mut views: Vec<VerifierOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_override_view(item));
    }
    views
}

pub fn sum_verifier_override_view(view: &VerifierOverrideView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_port as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_override("code-804", "title-309");
        let view = to_verifier_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
