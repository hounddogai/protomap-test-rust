use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal query variance.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceView {
    pub code: String,
    pub title_text: String,
    pub version_text: String,
    pub description: String,
    pub cluster_text: String,
    pub cost_text: String,
    pub name: String,
    pub visible: String,
    pub score: String,
    pub limit: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_seasonal_query_variance_view(item: &SeasonalQueryVariance) -> SeasonalQueryVarianceView {
    SeasonalQueryVarianceView {
        code: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 64),
        version_text: item.version.to_string(),
        description: item.description.clone(),
        cluster_text: item.cluster.clone(),
        cost_text: format!("{:.2}", item.cost),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        score: format!("{:.2}", item.score),
        limit: item.limit.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_query_variance_views(items: &[SeasonalQueryVariance]) -> Vec<SeasonalQueryVarianceView> {
    let mut views: Vec<SeasonalQueryVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_query_variance_view(item));
    }
    views
}

pub fn sum_seasonal_query_variance_view(view: &SeasonalQueryVarianceView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_version as f64 + parsed_cost + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_query_variance("code-462", "title-211");
        let view = to_seasonal_query_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_query_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
