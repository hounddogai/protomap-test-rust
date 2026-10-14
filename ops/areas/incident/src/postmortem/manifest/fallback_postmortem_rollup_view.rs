use super::*;

use acme_kit::text;

/// How screens and exports show a fallback postmortem rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollupView {
    pub code: String,
    pub name: String,
    pub grace_days_text: String,
    pub title: String,
    pub expires_at_text: String,
    pub tenant_id_text: String,
    pub quantity: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_fallback_postmortem_rollup_view(item: &FallbackPostmortemRollup) -> FallbackPostmortemRollupView {
    FallbackPostmortemRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        grace_days_text: item.grace_days.to_string(),
        title: text::truncate(&item.title, 80),
        expires_at_text: item.expires_at.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        quantity: item.quantity.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_postmortem_rollup_views(items: &[FallbackPostmortemRollup]) -> Vec<FallbackPostmortemRollupView> {
    let mut views: Vec<FallbackPostmortemRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_postmortem_rollup_view(item));
    }
    views
}

pub fn sum_fallback_postmortem_rollup_view(view: &FallbackPostmortemRollupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_expires_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_postmortem_rollup("code-176", "name-738");
        let view = to_fallback_postmortem_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_postmortem_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
