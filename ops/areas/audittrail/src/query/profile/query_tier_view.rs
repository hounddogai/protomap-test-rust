use super::*;

/// How screens and exports show a query tier.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryTierView {
    pub code: String,
    pub title: String,
    pub ratio_text: String,
    pub discount_text: String,
    pub cipher_text: String,
    pub cost_text: String,
    pub active: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_query_tier_view(item: &QueryTier) -> QueryTierView {
    QueryTierView {
        code: item.code.clone(),
        title: item.title.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        discount_text: format!("{:.2}", item.discount),
        cipher_text: item.cipher.clone(),
        cost_text: format!("{:.2}", item.cost),
        active: format!("{}", item.active),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_tier_views(items: &[QueryTier]) -> Vec<QueryTierView> {
    let mut views: Vec<QueryTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_tier_view(item));
    }
    views
}

pub fn sum_query_tier_view(view: &QueryTierView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ratio + parsed_discount + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_tier("code-292", "title-472");
        let view = to_query_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
