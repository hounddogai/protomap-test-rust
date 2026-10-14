use super::*;

/// How screens and exports show a postmortem entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEntryView {
    pub code: String,
    pub label: String,
    pub cost_text: String,
    pub category_text: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_postmortem_entry_view(item: &PostmortemEntry) -> PostmortemEntryView {
    PostmortemEntryView {
        code: item.code.clone(),
        label: item.label.clone(),
        cost_text: format!("{:.2}", item.cost),
        category_text: item.category.clone(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_entry_views(items: &[PostmortemEntry]) -> Vec<PostmortemEntryView> {
    let mut views: Vec<PostmortemEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_entry_view(item));
    }
    views
}

pub fn sum_postmortem_entry_view(view: &PostmortemEntryView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_entry("code-349", "label-844");
        let view = to_postmortem_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
