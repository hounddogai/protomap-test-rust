use super::*;

/// How screens and exports show a quote mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMappingView {
    pub code: String,
    pub title: String,
    pub quantity: String,
    pub phone: String,
    pub region: String,
    pub visible_text: String,
    pub timezone: String,
    pub archived: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_quote_mapping_view(item: &QuoteMapping) -> QuoteMappingView {
    QuoteMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        phone: item.phone.clone(),
        region: item.region.clone(),
        visible_text: format!("{}", item.visible),
        timezone: item.timezone.clone(),
        archived: format!("{}", item.archived),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_mapping_views(items: &[QuoteMapping]) -> Vec<QuoteMappingView> {
    let mut views: Vec<QuoteMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_mapping_view(item));
    }
    views
}

pub fn sum_quote_mapping_view(view: &QuoteMappingView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_mapping("code-813", "title-321");
        let view = to_quote_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
