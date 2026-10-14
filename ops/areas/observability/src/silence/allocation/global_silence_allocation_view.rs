use super::*;

use acme_kit::text;

/// How screens and exports show a global silence allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocationView {
    pub code: String,
    pub label: String,
    pub description_text: String,
    pub scope: String,
    pub ends_at_text: String,
    pub category: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_global_silence_allocation_view(item: &GlobalSilenceAllocation) -> GlobalSilenceAllocationView {
    GlobalSilenceAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        description_text: text::truncate(&item.description, 120),
        scope: item.scope.clone(),
        ends_at_text: item.ends_at.to_string(),
        category: item.category.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_silence_allocation_views(items: &[GlobalSilenceAllocation]) -> Vec<GlobalSilenceAllocationView> {
    let mut views: Vec<GlobalSilenceAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_silence_allocation_view(item));
    }
    views
}

pub fn sum_global_silence_allocation_view(view: &GlobalSilenceAllocationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_silence_allocation("code-933", "label-195");
        let view = to_global_silence_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_silence_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
