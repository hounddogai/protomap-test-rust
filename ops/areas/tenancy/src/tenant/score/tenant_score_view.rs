use super::*;

use acme_kit::text;

/// How screens and exports show a tenant score.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantScoreView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub country: String,
    pub name_text: String,
    pub status_label: String,
}

pub fn to_tenant_score_view(item: &TenantScore) -> TenantScoreView {
    TenantScoreView {
        code: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 40),
        score: format!("{:.2}", item.score),
        country: item.country.clone(),
        name_text: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_score_views(items: &[TenantScore]) -> Vec<TenantScoreView> {
    let mut views: Vec<TenantScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_score_view(item));
    }
    views
}

pub fn sum_tenant_score_view(view: &TenantScoreView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_score("code-679", "title-612");
        let view = to_tenant_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
