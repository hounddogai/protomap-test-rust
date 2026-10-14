use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTierView {
    pub code: String,
    pub name: String,
    pub principal_text: String,
    pub version: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_regional_usage_tier_view(item: &RegionalUsageTier) -> RegionalUsageTierView {
    RegionalUsageTierView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        principal_text: item.principal.clone(),
        version: item.version.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_tier_views(items: &[RegionalUsageTier]) -> Vec<RegionalUsageTierView> {
    let mut views: Vec<RegionalUsageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_tier_view(item));
    }
    views
}

pub fn sum_regional_usage_tier_view(view: &RegionalUsageTierView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_version as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_usage_tier("code-787", "name-468");
        let view = to_regional_usage_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_usage_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
