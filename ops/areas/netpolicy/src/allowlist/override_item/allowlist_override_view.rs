use super::*;

/// How screens and exports show a allowlist override.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverrideView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub version: String,
    pub ends_at: String,
    pub region: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_allowlist_override_view(item: &AllowlistOverride) -> AllowlistOverrideView {
    AllowlistOverrideView {
        code: item.code.clone(),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        version: item.version.to_string(),
        ends_at: item.ends_at.to_string(),
        region: item.region.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_override_views(items: &[AllowlistOverride]) -> Vec<AllowlistOverrideView> {
    let mut views: Vec<AllowlistOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_override_view(item));
    }
    views
}

pub fn sum_allowlist_override_view(view: &AllowlistOverrideView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_version as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_override("code-223", "title-800");
        let view = to_allowlist_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
