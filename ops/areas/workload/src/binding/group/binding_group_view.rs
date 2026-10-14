use super::*;

use acme_kit::text;

/// How screens and exports show a binding group.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingGroupView {
    pub code: String,
    pub label_text: String,
    pub cluster: String,
    pub batch_size_text: String,
    pub revision: String,
    pub cipher: String,
    pub notes: String,
    pub starts_at_text: String,
    pub owner_id: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_binding_group_view(item: &BindingGroup) -> BindingGroupView {
    BindingGroupView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        cluster: item.cluster.clone(),
        batch_size_text: item.batch_size.to_string(),
        revision: item.revision.to_string(),
        cipher: item.cipher.clone(),
        notes: item.notes.clone(),
        starts_at_text: item.starts_at.to_string(),
        owner_id: text::truncate(&item.owner_id, 64),
        reviewer_text: text::truncate(&item.reviewer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_group_views(items: &[BindingGroup]) -> Vec<BindingGroupView> {
    let mut views: Vec<BindingGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_group_view(item));
    }
    views
}

pub fn sum_binding_group_view(view: &BindingGroupView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_revision as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_group("code-407", "label-424");
        let view = to_binding_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
