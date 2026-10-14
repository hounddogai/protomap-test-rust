use super::*;

use acme_kit::text;

/// How screens and exports show a regional retention group.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionGroupView {
    pub code: String,
    pub label: String,
    pub version: String,
    pub created_at: String,
    pub minimum: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_regional_retention_group_view(item: &RegionalRetentionGroup) -> RegionalRetentionGroupView {
    RegionalRetentionGroupView {
        code: item.code.clone(),
        label: item.label.clone(),
        version: item.version.to_string(),
        created_at: item.created_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        channel: text::truncate(&item.channel, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_retention_group_views(items: &[RegionalRetentionGroup]) -> Vec<RegionalRetentionGroupView> {
    let mut views: Vec<RegionalRetentionGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_retention_group_view(item));
    }
    views
}

pub fn sum_regional_retention_group_view(view: &RegionalRetentionGroupView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_version as f64 + (parsed_created_at as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_retention_group("code-498", "label-197");
        let view = to_regional_retention_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_retention_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
