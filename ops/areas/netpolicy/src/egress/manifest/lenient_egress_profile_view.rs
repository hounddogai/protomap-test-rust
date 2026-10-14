use super::*;

use acme_kit::text;

/// How screens and exports show a lenient egress profile.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressProfileView {
    pub code: String,
    pub name: String,
    pub version_text: String,
    pub owner_id_text: String,
    pub fingerprint_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_lenient_egress_profile_view(item: &LenientEgressProfile) -> LenientEgressProfileView {
    LenientEgressProfileView {
        code: item.code.clone(),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        owner_id_text: item.owner_id.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 40),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_egress_profile_views(items: &[LenientEgressProfile]) -> Vec<LenientEgressProfileView> {
    let mut views: Vec<LenientEgressProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_egress_profile_view(item));
    }
    views
}

pub fn sum_lenient_egress_profile_view(view: &LenientEgressProfileView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_egress_profile("code-694", "name-131");
        let view = to_lenient_egress_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_egress_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
