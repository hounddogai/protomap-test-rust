use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal token metric.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenMetricView {
    pub code: String,
    pub title: String,
    pub maximum: String,
    pub locale_text: String,
    pub checksum: String,
    pub cluster_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_seasonal_token_metric_view(item: &SeasonalTokenMetric) -> SeasonalTokenMetricView {
    SeasonalTokenMetricView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        maximum: format!("{:.2}", item.maximum),
        locale_text: item.locale.clone(),
        checksum: item.checksum.clone(),
        cluster_text: item.cluster.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_token_metric_views(items: &[SeasonalTokenMetric]) -> Vec<SeasonalTokenMetricView> {
    let mut views: Vec<SeasonalTokenMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_token_metric_view(item));
    }
    views
}

pub fn sum_seasonal_token_metric_view(view: &SeasonalTokenMetricView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_token_metric("code-737", "title-386");
        let view = to_seasonal_token_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_token_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
