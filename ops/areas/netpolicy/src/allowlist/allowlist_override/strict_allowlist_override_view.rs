use super::*;

use acme_kit::text;

/// How screens and exports show a strict allowlist override.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistOverrideView {
    pub code: String,
    pub name_text: String,
    pub unit: String,
    pub cipher: String,
    pub summary_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_strict_allowlist_override_view(item: &StrictAllowlistOverride) -> StrictAllowlistOverrideView {
    StrictAllowlistOverrideView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        unit: item.unit.clone(),
        cipher: item.cipher.clone(),
        summary_text: item.summary.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_allowlist_override_views(items: &[StrictAllowlistOverride]) -> Vec<StrictAllowlistOverrideView> {
    let mut views: Vec<StrictAllowlistOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_allowlist_override_view(item));
    }
    views
}

pub fn sum_strict_allowlist_override_view(view: &StrictAllowlistOverrideView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_allowlist_override("code-73", "name-801");
        let view = to_strict_allowlist_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_allowlist_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
