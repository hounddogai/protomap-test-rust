use super::*;

use acme_kit::text;

/// How screens and exports show a probe tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTierView {
    pub code: String,
    pub title: String,
    pub quota_units_text: String,
    pub version_text: String,
    pub audience: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_probe_tier_view(item: &ProbeTier) -> ProbeTierView {
    ProbeTierView {
        code: item.code.clone(),
        title: item.title.clone(),
        quota_units_text: item.quota_units.to_string(),
        version_text: item.version.to_string(),
        audience: text::truncate(&item.audience, 120),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_tier_views(items: &[ProbeTier]) -> Vec<ProbeTierView> {
    let mut views: Vec<ProbeTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_tier_view(item));
    }
    views
}

pub fn sum_probe_tier_view(view: &ProbeTierView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_tier("code-767", "title-756");
        let view = to_probe_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
