use super::*;

use acme_kit::text;

/// How screens and exports show a pending binding group.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroupView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub ceiling: String,
    pub channel_text: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_pending_binding_group_view(item: &PendingBindingGroup) -> PendingBindingGroupView {
    PendingBindingGroupView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        notes: item.notes.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        channel_text: text::truncate(&item.channel, 80),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_binding_group_views(items: &[PendingBindingGroup]) -> Vec<PendingBindingGroupView> {
    let mut views: Vec<PendingBindingGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_binding_group_view(item));
    }
    views
}

pub fn sum_pending_binding_group_view(view: &PendingBindingGroupView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ceiling + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_binding_group("code-420", "name-315");
        let view = to_pending_binding_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_binding_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
