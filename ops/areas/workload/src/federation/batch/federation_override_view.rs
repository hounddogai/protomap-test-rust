use super::*;

use acme_kit::text;

/// How screens and exports show a federation override.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideView {
    pub code: String,
    pub title_text: String,
    pub audience_text: String,
    pub limit: String,
    pub weight_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_federation_override_view(item: &FederationOverride) -> FederationOverrideView {
    FederationOverrideView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        audience_text: text::truncate(&item.audience, 64),
        limit: item.limit.to_string(),
        weight_text: format!("{:.2}", item.weight),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_override_views(items: &[FederationOverride]) -> Vec<FederationOverrideView> {
    let mut views: Vec<FederationOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_override_view(item));
    }
    views
}

pub fn sum_federation_override_view(view: &FederationOverrideView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_limit as f64 + parsed_weight + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_override("code-913", "title-258");
        let view = to_federation_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
