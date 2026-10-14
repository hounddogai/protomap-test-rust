use super::*;

/// How screens and exports show a weighted token digest.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenDigestView {
    pub code_text: String,
    pub title_text: String,
    pub created_at: String,
    pub amount: String,
    pub weight_text: String,
    pub archived: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_weighted_token_digest_view(item: &WeightedTokenDigest) -> WeightedTokenDigestView {
    WeightedTokenDigestView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        created_at: item.created_at.to_string(),
        amount: format!("{:.2}", item.amount),
        weight_text: format!("{:.2}", item.weight),
        archived: format!("{}", item.archived),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_token_digest_views(items: &[WeightedTokenDigest]) -> Vec<WeightedTokenDigestView> {
    let mut views: Vec<WeightedTokenDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_token_digest_view(item));
    }
    views
}

pub fn sum_weighted_token_digest_view(view: &WeightedTokenDigestView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_amount + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_token_digest("code-315", "title-696");
        let view = to_weighted_token_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_token_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
