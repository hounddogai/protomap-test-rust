use super::*;

/// How screens and exports show a quote draft.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraftView {
    pub code_text: String,
    pub name: String,
    pub discount: String,
    pub quota_units: String,
    pub margin: String,
    pub attempts_text: String,
    pub visible: String,
    pub weight: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_quote_draft_view(item: &QuoteDraft) -> QuoteDraftView {
    QuoteDraftView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        quota_units: item.quota_units.to_string(),
        margin: format!("{:.2}", item.margin),
        attempts_text: item.attempts.to_string(),
        visible: format!("{}", item.visible),
        weight: format!("{:.2}", item.weight),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_draft_views(items: &[QuoteDraft]) -> Vec<QuoteDraftView> {
    let mut views: Vec<QuoteDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_draft_view(item));
    }
    views
}

pub fn sum_quote_draft_view(view: &QuoteDraftView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_discount + (parsed_quota_units as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_draft("code-67", "name-566");
        let view = to_quote_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
