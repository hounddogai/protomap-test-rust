use super::*;

use acme_kit::text;

/// How screens and exports show a zone template.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTemplateView {
    pub code: String,
    pub label: String,
    pub category: String,
    pub visible_text: String,
    pub tenant_id: String,
    pub locale: String,
    pub archived: String,
    pub amount: String,
    pub currency: String,
    pub ratio: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_zone_template_view(item: &ZoneTemplate) -> ZoneTemplateView {
    ZoneTemplateView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        category: item.category.clone(),
        visible_text: format!("{}", item.visible),
        tenant_id: item.tenant_id.clone(),
        locale: item.locale.clone(),
        archived: format!("{}", item.archived),
        amount: format!("{:.2}", item.amount),
        currency: item.currency.clone(),
        ratio: format!("{:.2}", item.ratio),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_template_views(items: &[ZoneTemplate]) -> Vec<ZoneTemplateView> {
    let mut views: Vec<ZoneTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_template_view(item));
    }
    views
}

pub fn sum_zone_template_view(view: &ZoneTemplateView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_amount + parsed_ratio + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_template("code-795", "label-85");
        let view = to_zone_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
