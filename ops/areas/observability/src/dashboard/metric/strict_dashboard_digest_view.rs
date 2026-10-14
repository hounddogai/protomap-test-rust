use super::*;

use acme_kit::text;

/// How screens and exports show a strict dashboard digest.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigestView {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub fingerprint: String,
    pub floor_value: String,
    pub owner_id: String,
    pub burst: String,
    pub rate: String,
    pub principal: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_strict_dashboard_digest_view(item: &StrictDashboardDigest) -> StrictDashboardDigestView {
    StrictDashboardDigestView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        checksum: item.checksum.clone(),
        fingerprint: item.fingerprint.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        owner_id: item.owner_id.clone(),
        burst: item.burst.to_string(),
        rate: format!("{:.2}", item.rate),
        principal: item.principal.clone(),
        issuer: text::truncate(&item.issuer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_dashboard_digest_views(items: &[StrictDashboardDigest]) -> Vec<StrictDashboardDigestView> {
    let mut views: Vec<StrictDashboardDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_dashboard_digest_view(item));
    }
    views
}

pub fn sum_strict_dashboard_digest_view(view: &StrictDashboardDigestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_floor_value + (parsed_burst as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_dashboard_digest("code-694", "title-582");
        let view = to_strict_dashboard_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_dashboard_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
