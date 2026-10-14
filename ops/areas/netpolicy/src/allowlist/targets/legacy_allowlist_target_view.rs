use super::*;

use acme_kit::text;

/// How screens and exports show a legacy allowlist target.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTargetView {
    pub code: String,
    pub title: String,
    pub maximum: String,
    pub segment: String,
    pub ceiling: String,
    pub currency: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_legacy_allowlist_target_view(item: &LegacyAllowlistTarget) -> LegacyAllowlistTargetView {
    LegacyAllowlistTargetView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        segment: item.segment.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        currency: text::truncate(&item.currency, 64),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_allowlist_target_views(items: &[LegacyAllowlistTarget]) -> Vec<LegacyAllowlistTargetView> {
    let mut views: Vec<LegacyAllowlistTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_allowlist_target_view(item));
    }
    views
}

pub fn sum_legacy_allowlist_target_view(view: &LegacyAllowlistTargetView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_maximum + parsed_ceiling + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_allowlist_target("code-802", "title-220");
        let view = to_legacy_allowlist_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_allowlist_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
