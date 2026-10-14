use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal token baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenBaselineView {
    pub code: String,
    pub name: String,
    pub amount_text: String,
    pub cipher: String,
    pub ceiling_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_seasonal_token_baseline_view(item: &SeasonalTokenBaseline) -> SeasonalTokenBaselineView {
    SeasonalTokenBaselineView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        amount_text: format!("{:.2}", item.amount),
        cipher: item.cipher.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_token_baseline_views(items: &[SeasonalTokenBaseline]) -> Vec<SeasonalTokenBaselineView> {
    let mut views: Vec<SeasonalTokenBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_token_baseline_view(item));
    }
    views
}

pub fn sum_seasonal_token_baseline_view(view: &SeasonalTokenBaselineView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_amount + parsed_ceiling + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_token_baseline("code-787", "name-502");
        let view = to_seasonal_token_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_token_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
