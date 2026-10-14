use super::*;

use acme_kit::text;

/// How screens and exports show a capped probe exception.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionView {
    pub code: String,
    pub title_text: String,
    pub revision: String,
    pub rotation_days_text: String,
    pub limit: String,
    pub issuer: String,
    pub contact_email: String,
    pub label_text: String,
    pub retries_text: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_capped_probe_exception_view(item: &CappedProbeException) -> CappedProbeExceptionView {
    CappedProbeExceptionView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        revision: item.revision.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        limit: item.limit.to_string(),
        issuer: item.issuer.clone(),
        contact_email: item.contact_email.clone(),
        label_text: item.label.clone(),
        retries_text: item.retries.to_string(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_probe_exception_views(items: &[CappedProbeException]) -> Vec<CappedProbeExceptionView> {
    let mut views: Vec<CappedProbeExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_probe_exception_view(item));
    }
    views
}

pub fn sum_capped_probe_exception_view(view: &CappedProbeExceptionView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_rotation_days as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_probe_exception("code-3", "title-525");
        let view = to_capped_probe_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_probe_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
