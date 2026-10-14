use super::*;

/// How screens and exports show a fallback nonce estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceEstimateView {
    pub code: String,
    pub title: String,
    pub margin: String,
    pub quantity: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_fallback_nonce_estimate_view(item: &FallbackNonceEstimate) -> FallbackNonceEstimateView {
    FallbackNonceEstimateView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        quantity: item.quantity.to_string(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_nonce_estimate_views(items: &[FallbackNonceEstimate]) -> Vec<FallbackNonceEstimateView> {
    let mut views: Vec<FallbackNonceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_nonce_estimate_view(item));
    }
    views
}

pub fn sum_fallback_nonce_estimate_view(view: &FallbackNonceEstimateView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_margin + (parsed_quantity as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_nonce_estimate("code-612", "title-530");
        let view = to_fallback_nonce_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_nonce_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
