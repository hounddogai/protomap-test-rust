use super::*;

use acme_kit::text;

/// How screens and exports show a primary verifier override.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverrideView {
    pub code_text: String,
    pub title: String,
    pub region: String,
    pub ceiling_text: String,
    pub ends_at: String,
    pub score: String,
    pub quantity: String,
    pub label: String,
    pub expires_at_text: String,
    pub country: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_primary_verifier_override_view(item: &PrimaryVerifierOverride) -> PrimaryVerifierOverrideView {
    PrimaryVerifierOverrideView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        region: item.region.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        ends_at: item.ends_at.to_string(),
        score: format!("{:.2}", item.score),
        quantity: item.quantity.to_string(),
        label: text::truncate(&item.label, 80),
        expires_at_text: item.expires_at.to_string(),
        country: item.country.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_verifier_override_views(items: &[PrimaryVerifierOverride]) -> Vec<PrimaryVerifierOverrideView> {
    let mut views: Vec<PrimaryVerifierOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_verifier_override_view(item));
    }
    views
}

pub fn sum_primary_verifier_override_view(view: &PrimaryVerifierOverrideView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ceiling + (parsed_ends_at as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_verifier_override("code-270", "title-837");
        let view = to_primary_verifier_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_verifier_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
