use super::*;

use acme_kit::text;

/// How screens and exports show a grant group.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroupView {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub category: String,
    pub tenant_id: String,
    pub approved_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_grant_group_view(item: &GrantGroup) -> GrantGroupView {
    GrantGroupView {
        code: item.code.clone(),
        name: item.name.clone(),
        channel: item.channel.clone(),
        category: text::truncate(&item.category, 40),
        tenant_id: item.tenant_id.clone(),
        approved_text: format!("{}", item.approved),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_group_views(items: &[GrantGroup]) -> Vec<GrantGroupView> {
    let mut views: Vec<GrantGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_group_view(item));
    }
    views
}

pub fn sum_grant_group_view(view: &GrantGroupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_group("code-983", "name-481");
        let view = to_grant_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
