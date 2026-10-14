use super::*;

use acme_kit::text;

/// How screens and exports show a grant window.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantWindowView {
    pub code: String,
    pub title_text: String,
    pub locale: String,
    pub minimum_text: String,
    pub tenant_id: String,
    pub issuer: String,
    pub source: String,
    pub ceiling: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_grant_window_view(item: &GrantWindow) -> GrantWindowView {
    GrantWindowView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        locale: text::truncate(&item.locale, 120),
        minimum_text: format!("{:.2}", item.minimum),
        tenant_id: text::truncate(&item.tenant_id, 120),
        issuer: item.issuer.clone(),
        source: item.source.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        principal: text::truncate(&item.principal, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_window_views(items: &[GrantWindow]) -> Vec<GrantWindowView> {
    let mut views: Vec<GrantWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_window_view(item));
    }
    views
}

pub fn sum_grant_window_view(view: &GrantWindowView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_minimum + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_window("code-680", "title-353");
        let view = to_grant_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
