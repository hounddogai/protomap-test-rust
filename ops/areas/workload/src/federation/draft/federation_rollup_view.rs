use super::*;

/// How screens and exports show a federation rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupView {
    pub code: String,
    pub label_text: String,
    pub window_days: String,
    pub checksum: String,
    pub contact_email: String,
    pub active: String,
    pub category: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_federation_rollup_view(item: &FederationRollup) -> FederationRollupView {
    FederationRollupView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        window_days: item.window_days.to_string(),
        checksum: item.checksum.clone(),
        contact_email: item.contact_email.clone(),
        active: format!("{}", item.active),
        category: item.category.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_rollup_views(items: &[FederationRollup]) -> Vec<FederationRollupView> {
    let mut views: Vec<FederationRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_rollup_view(item));
    }
    views
}

pub fn sum_federation_rollup_view(view: &FederationRollupView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_rollup("code-370", "label-132");
        let view = to_federation_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
