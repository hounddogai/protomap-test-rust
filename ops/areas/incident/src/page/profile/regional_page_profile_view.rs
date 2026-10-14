use super::*;

use acme_kit::text;

/// How screens and exports show a regional page profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfileView {
    pub code_text: String,
    pub title: String,
    pub maximum: String,
    pub grace_days: String,
    pub cost_text: String,
    pub key_id: String,
    pub window_days: String,
    pub tenant_id: String,
    pub quantity_text: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_regional_page_profile_view(item: &RegionalPageProfile) -> RegionalPageProfileView {
    RegionalPageProfileView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        grace_days: item.grace_days.to_string(),
        cost_text: format!("{:.2}", item.cost),
        key_id: item.key_id.clone(),
        window_days: item.window_days.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        quantity_text: item.quantity.to_string(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_page_profile_views(items: &[RegionalPageProfile]) -> Vec<RegionalPageProfileView> {
    let mut views: Vec<RegionalPageProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_page_profile_view(item));
    }
    views
}

pub fn sum_regional_page_profile_view(view: &RegionalPageProfileView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_maximum + (parsed_grace_days as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_page_profile("code-439", "title-309");
        let view = to_regional_page_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_page_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
