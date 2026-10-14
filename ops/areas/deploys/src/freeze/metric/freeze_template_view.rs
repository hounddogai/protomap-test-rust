use super::*;

use acme_kit::text;

/// How screens and exports show a freeze template.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplateView {
    pub code: String,
    pub label: String,
    pub burst_text: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_freeze_template_view(item: &FreezeTemplate) -> FreezeTemplateView {
    FreezeTemplateView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        burst_text: item.burst.to_string(),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_template_views(items: &[FreezeTemplate]) -> Vec<FreezeTemplateView> {
    let mut views: Vec<FreezeTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_template_view(item));
    }
    views
}

pub fn sum_freeze_template_view(view: &FreezeTemplateView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_template("code-486", "label-159");
        let view = to_freeze_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
