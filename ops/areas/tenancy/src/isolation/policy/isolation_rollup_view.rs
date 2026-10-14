use super::*;

use acme_kit::text;

/// How screens and exports show a isolation rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRollupView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub expires_at_text: String,
    pub quantity: String,
    pub name: String,
    pub issuer_text: String,
    pub minimum: String,
    pub starts_at_text: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_isolation_rollup_view(item: &IsolationRollup) -> IsolationRollupView {
    IsolationRollupView {
        code: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 120),
        country: text::truncate(&item.country, 64),
        expires_at_text: item.expires_at.to_string(),
        quantity: item.quantity.to_string(),
        name: item.name.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        minimum: format!("{:.2}", item.minimum),
        starts_at_text: item.starts_at.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_rollup_views(items: &[IsolationRollup]) -> Vec<IsolationRollupView> {
    let mut views: Vec<IsolationRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_rollup_view(item));
    }
    views
}

pub fn sum_isolation_rollup_view(view: &IsolationRollupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_quantity as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_rollup("code-723", "title-360");
        let view = to_isolation_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
