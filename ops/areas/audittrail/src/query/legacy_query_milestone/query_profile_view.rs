use super::*;

/// How screens and exports show a query profile.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfileView {
    pub code: String,
    pub name: String,
    pub region_text: String,
    pub expires_at: String,
    pub discount: String,
    pub scope: String,
    pub quota_units: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_query_profile_view(item: &QueryProfile) -> QueryProfileView {
    QueryProfileView {
        code: item.code.clone(),
        name: item.name.clone(),
        region_text: item.region.clone(),
        expires_at: item.expires_at.to_string(),
        discount: format!("{:.2}", item.discount),
        scope: item.scope.clone(),
        quota_units: item.quota_units.to_string(),
        title_text: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_profile_views(items: &[QueryProfile]) -> Vec<QueryProfileView> {
    let mut views: Vec<QueryProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_profile_view(item));
    }
    views
}

pub fn sum_query_profile_view(view: &QueryProfileView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_discount + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_profile("code-945", "name-952");
        let view = to_query_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
