use super::*;

use acme_kit::text;

/// How screens and exports show a suspension threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThresholdView {
    pub code: String,
    pub name: String,
    pub timezone_text: String,
    pub currency_text: String,
    pub issuer: String,
    pub retries_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_suspension_threshold_view(item: &SuspensionThreshold) -> SuspensionThresholdView {
    SuspensionThresholdView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 80),
        timezone_text: text::truncate(&item.timezone, 40),
        currency_text: item.currency.clone(),
        issuer: item.issuer.clone(),
        retries_text: item.retries.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_threshold_views(items: &[SuspensionThreshold]) -> Vec<SuspensionThresholdView> {
    let mut views: Vec<SuspensionThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_threshold_view(item));
    }
    views
}

pub fn sum_suspension_threshold_view(view: &SuspensionThresholdView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_retries as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_threshold("code-363", "name-656");
        let view = to_suspension_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
