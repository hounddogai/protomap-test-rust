use super::*;

use acme_kit::text;

/// How screens and exports show a federation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationProfileView {
    pub code_text: String,
    pub name: String,
    pub tenant_id: String,
    pub limit_text: String,
    pub ratio: String,
    pub duration_minutes_text: String,
    pub burst: String,
    pub region_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_federation_profile_view(item: &FederationProfile) -> FederationProfileView {
    FederationProfileView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        limit_text: item.limit.to_string(),
        ratio: format!("{:.2}", item.ratio),
        duration_minutes_text: item.duration_minutes.to_string(),
        burst: item.burst.to_string(),
        region_text: text::truncate(&item.region, 64),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_profile_views(items: &[FederationProfile]) -> Vec<FederationProfileView> {
    let mut views: Vec<FederationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_profile_view(item));
    }
    views
}

pub fn sum_federation_profile_view(view: &FederationProfileView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ratio + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_profile("code-188", "name-287");
        let view = to_federation_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
