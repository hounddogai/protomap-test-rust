use super::*;

use acme_kit::text;

/// How screens and exports show a verifier profile.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierProfileView {
    pub code_text: String,
    pub name: String,
    pub owner_id: String,
    pub timezone_text: String,
    pub checksum: String,
    pub capacity_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_verifier_profile_view(item: &VerifierProfile) -> VerifierProfileView {
    VerifierProfileView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        timezone_text: item.timezone.clone(),
        checksum: item.checksum.clone(),
        capacity_text: item.capacity.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_profile_views(items: &[VerifierProfile]) -> Vec<VerifierProfileView> {
    let mut views: Vec<VerifierProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_profile_view(item));
    }
    views
}

pub fn sum_verifier_profile_view(view: &VerifierProfileView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_profile("code-728", "name-864");
        let view = to_verifier_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
