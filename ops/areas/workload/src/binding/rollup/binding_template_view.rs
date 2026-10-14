use super::*;

/// How screens and exports show a binding template.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplateView {
    pub code_text: String,
    pub title: String,
    pub audience: String,
    pub segment: String,
    pub active: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_binding_template_view(item: &BindingTemplate) -> BindingTemplateView {
    BindingTemplateView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        audience: item.audience.clone(),
        segment: item.segment.clone(),
        active: format!("{}", item.active),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_template_views(items: &[BindingTemplate]) -> Vec<BindingTemplateView> {
    let mut views: Vec<BindingTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_template_view(item));
    }
    views
}

pub fn sum_binding_template_view(view: &BindingTemplateView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_template("code-18", "title-533");
        let view = to_binding_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
