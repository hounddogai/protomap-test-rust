use super::*;

use acme_kit::text;

/// How screens and exports show a usage variance.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageVarianceView {
    pub code_text: String,
    pub name: String,
    pub issuer_text: String,
    pub region: String,
    pub notes: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_usage_variance_view(item: &UsageVariance) -> UsageVarianceView {
    UsageVarianceView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        issuer_text: item.issuer.clone(),
        region: item.region.clone(),
        notes: text::truncate(&item.notes, 120),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_variance_views(items: &[UsageVariance]) -> Vec<UsageVarianceView> {
    let mut views: Vec<UsageVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_variance_view(item));
    }
    views
}

pub fn sum_usage_variance_view(view: &UsageVarianceView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_variance("code-690", "name-751");
        let view = to_usage_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
