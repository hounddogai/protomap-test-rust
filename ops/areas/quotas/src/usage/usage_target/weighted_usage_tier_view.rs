use super::*;

/// How screens and exports show a weighted usage tier.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTierView {
    pub code: String,
    pub title_text: String,
    pub quantity: String,
    pub name: String,
    pub rate: String,
    pub score: String,
    pub starts_at_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_weighted_usage_tier_view(item: &WeightedUsageTier) -> WeightedUsageTierView {
    WeightedUsageTierView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quantity: item.quantity.to_string(),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        score: format!("{:.2}", item.score),
        starts_at_text: item.starts_at.to_string(),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_usage_tier_views(items: &[WeightedUsageTier]) -> Vec<WeightedUsageTierView> {
    let mut views: Vec<WeightedUsageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_usage_tier_view(item));
    }
    views
}

pub fn sum_weighted_usage_tier_view(view: &WeightedUsageTierView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_rate + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_usage_tier("code-332", "title-361");
        let view = to_weighted_usage_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_usage_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
