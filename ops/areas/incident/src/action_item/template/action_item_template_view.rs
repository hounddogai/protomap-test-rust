use super::*;

use acme_kit::text;

/// How screens and exports show a action item template.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplateView {
    pub code: String,
    pub title: String,
    pub locale_text: String,
    pub pinned: String,
    pub external_ref: String,
    pub owner_id: String,
    pub discount: String,
    pub limit: String,
    pub scope: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_action_item_template_view(item: &ActionItemTemplate) -> ActionItemTemplateView {
    ActionItemTemplateView {
        code: item.code.clone(),
        title: item.title.clone(),
        locale_text: text::truncate(&item.locale, 120),
        pinned: format!("{}", item.pinned),
        external_ref: text::truncate(&item.external_ref, 40),
        owner_id: item.owner_id.clone(),
        discount: format!("{:.2}", item.discount),
        limit: item.limit.to_string(),
        scope: item.scope.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_template_views(items: &[ActionItemTemplate]) -> Vec<ActionItemTemplateView> {
    let mut views: Vec<ActionItemTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_template_view(item));
    }
    views
}

pub fn sum_action_item_template_view(view: &ActionItemTemplateView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_discount + (parsed_limit as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_template("code-767", "title-701");
        let view = to_action_item_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
