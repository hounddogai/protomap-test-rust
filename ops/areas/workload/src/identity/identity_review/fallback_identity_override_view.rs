use super::*;

/// How screens and exports show a fallback identity override.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverrideView {
    pub code_text: String,
    pub name_text: String,
    pub starts_at: String,
    pub capacity: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_fallback_identity_override_view(item: &FallbackIdentityOverride) -> FallbackIdentityOverrideView {
    FallbackIdentityOverrideView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        capacity: item.capacity.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_identity_override_views(items: &[FallbackIdentityOverride]) -> Vec<FallbackIdentityOverrideView> {
    let mut views: Vec<FallbackIdentityOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_identity_override_view(item));
    }
    views
}

pub fn sum_fallback_identity_override_view(view: &FallbackIdentityOverrideView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_capacity as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_identity_override("code-35", "name-987");
        let view = to_fallback_identity_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_identity_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
