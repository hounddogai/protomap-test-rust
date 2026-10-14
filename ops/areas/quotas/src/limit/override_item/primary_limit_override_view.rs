use super::*;

use acme_kit::text;

/// How screens and exports show a primary limit override.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideView {
    pub code: String,
    pub name: String,
    pub burst_text: String,
    pub locale: String,
    pub ratio_text: String,
    pub ttl_seconds_text: String,
    pub maximum: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_primary_limit_override_view(item: &PrimaryLimitOverride) -> PrimaryLimitOverrideView {
    PrimaryLimitOverrideView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        burst_text: item.burst.to_string(),
        locale: item.locale.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        maximum: format!("{:.2}", item.maximum),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_limit_override_views(items: &[PrimaryLimitOverride]) -> Vec<PrimaryLimitOverrideView> {
    let mut views: Vec<PrimaryLimitOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_limit_override_view(item));
    }
    views
}

pub fn sum_primary_limit_override_view(view: &PrimaryLimitOverrideView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_burst as f64 + parsed_ratio + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_limit_override("code-580", "name-532");
        let view = to_primary_limit_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_limit_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
