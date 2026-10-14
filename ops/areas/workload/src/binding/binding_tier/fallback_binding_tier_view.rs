use super::*;

/// How screens and exports show a fallback binding tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierView {
    pub code: String,
    pub label: String,
    pub window_days: String,
    pub external_ref_text: String,
    pub updated_at_text: String,
    pub archived: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_fallback_binding_tier_view(item: &FallbackBindingTier) -> FallbackBindingTierView {
    FallbackBindingTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        window_days: item.window_days.to_string(),
        external_ref_text: item.external_ref.clone(),
        updated_at_text: item.updated_at.to_string(),
        archived: format!("{}", item.archived),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_binding_tier_views(items: &[FallbackBindingTier]) -> Vec<FallbackBindingTierView> {
    let mut views: Vec<FallbackBindingTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_binding_tier_view(item));
    }
    views
}

pub fn sum_fallback_binding_tier_view(view: &FallbackBindingTierView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_updated_at as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_binding_tier("code-666", "label-125");
        let view = to_fallback_binding_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_binding_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
