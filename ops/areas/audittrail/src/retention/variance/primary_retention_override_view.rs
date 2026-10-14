use super::*;

/// How screens and exports show a primary retention override.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionOverrideView {
    pub code_text: String,
    pub title: String,
    pub ceiling: String,
    pub burst: String,
    pub approved: String,
    pub expires_at_text: String,
    pub margin_text: String,
    pub label: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_primary_retention_override_view(item: &PrimaryRetentionOverride) -> PrimaryRetentionOverrideView {
    PrimaryRetentionOverrideView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        burst: item.burst.to_string(),
        approved: format!("{}", item.approved),
        expires_at_text: item.expires_at.to_string(),
        margin_text: format!("{:.2}", item.margin),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_retention_override_views(items: &[PrimaryRetentionOverride]) -> Vec<PrimaryRetentionOverrideView> {
    let mut views: Vec<PrimaryRetentionOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_retention_override_view(item));
    }
    views
}

pub fn sum_primary_retention_override_view(view: &PrimaryRetentionOverrideView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ceiling + (parsed_burst as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_retention_override("code-420", "title-542");
        let view = to_primary_retention_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_retention_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
