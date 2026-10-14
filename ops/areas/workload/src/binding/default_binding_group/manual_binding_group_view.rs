use super::*;

use acme_kit::text;

/// How screens and exports show a manual binding group.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupView {
    pub code: String,
    pub name: String,
    pub title_text: String,
    pub maximum_text: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_manual_binding_group_view(item: &ManualBindingGroup) -> ManualBindingGroupView {
    ManualBindingGroupView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        title_text: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_binding_group_views(items: &[ManualBindingGroup]) -> Vec<ManualBindingGroupView> {
    let mut views: Vec<ManualBindingGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_binding_group_view(item));
    }
    views
}

pub fn sum_manual_binding_group_view(view: &ManualBindingGroupView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_maximum + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_binding_group("code-477", "name-247");
        let view = to_manual_binding_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_binding_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
