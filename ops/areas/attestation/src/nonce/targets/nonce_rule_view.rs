use super::*;

use acme_kit::text;

/// How screens and exports show a nonce rule.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRuleView {
    pub code: String,
    pub title_text: String,
    pub capacity_text: String,
    pub issuer: String,
    pub contact_email: String,
    pub rate: String,
    pub margin: String,
    pub external_ref_text: String,
    pub rotation_days: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_nonce_rule_view(item: &NonceRule) -> NonceRuleView {
    NonceRuleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        capacity_text: item.capacity.to_string(),
        issuer: text::truncate(&item.issuer, 40),
        contact_email: item.contact_email.clone(),
        rate: format!("{:.2}", item.rate),
        margin: format!("{:.2}", item.margin),
        external_ref_text: item.external_ref.clone(),
        rotation_days: item.rotation_days.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_rule_views(items: &[NonceRule]) -> Vec<NonceRuleView> {
    let mut views: Vec<NonceRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_rule_view(item));
    }
    views
}

pub fn sum_nonce_rule_view(view: &NonceRuleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_rate + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_rule("code-295", "title-107");
        let view = to_nonce_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
