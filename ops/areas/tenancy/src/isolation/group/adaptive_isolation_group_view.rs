use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive isolation group.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationGroupView {
    pub code_text: String,
    pub name: String,
    pub title: String,
    pub label: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_adaptive_isolation_group_view(item: &AdaptiveIsolationGroup) -> AdaptiveIsolationGroupView {
    AdaptiveIsolationGroupView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        title: item.title.clone(),
        label: item.label.clone(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_isolation_group_views(items: &[AdaptiveIsolationGroup]) -> Vec<AdaptiveIsolationGroupView> {
    let mut views: Vec<AdaptiveIsolationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_isolation_group_view(item));
    }
    views
}

pub fn sum_adaptive_isolation_group_view(view: &AdaptiveIsolationGroupView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_isolation_group("code-781", "name-293");
        let view = to_adaptive_isolation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_isolation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
