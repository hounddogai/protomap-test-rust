use super::*;

use acme_kit::text;

/// How screens and exports show a isolation group.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroupView {
    pub code_text: String,
    pub label: String,
    pub cost: String,
    pub updated_at_text: String,
    pub minimum: String,
    pub revision: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_isolation_group_view(item: &IsolationGroup) -> IsolationGroupView {
    IsolationGroupView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        updated_at_text: item.updated_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        revision: item.revision.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_group_views(items: &[IsolationGroup]) -> Vec<IsolationGroupView> {
    let mut views: Vec<IsolationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_group_view(item));
    }
    views
}

pub fn sum_isolation_group_view(view: &IsolationGroupView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_cost + (parsed_updated_at as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_group("code-195", "label-227");
        let view = to_isolation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
