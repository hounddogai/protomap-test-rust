use super::*;

use acme_kit::text;

/// How screens and exports show a isolation override.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideView {
    pub code: String,
    pub title: String,
    pub capacity: String,
    pub updated_at: String,
    pub issuer_text: String,
    pub unit: String,
    pub namespace: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_isolation_override_view(item: &IsolationOverride) -> IsolationOverrideView {
    IsolationOverrideView {
        code: item.code.clone(),
        title: item.title.clone(),
        capacity: item.capacity.to_string(),
        updated_at: item.updated_at.to_string(),
        issuer_text: item.issuer.clone(),
        unit: item.unit.clone(),
        namespace: text::truncate(&item.namespace, 64),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_override_views(items: &[IsolationOverride]) -> Vec<IsolationOverrideView> {
    let mut views: Vec<IsolationOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_override_view(item));
    }
    views
}

pub fn sum_isolation_override_view(view: &IsolationOverrideView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_override("code-738", "title-624");
        let view = to_isolation_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
