use super::*;

use acme_kit::text;

/// How screens and exports show a primary limit adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAdjustmentView {
    pub code: String,
    pub name: String,
    pub ends_at_text: String,
    pub host: String,
    pub updated_at: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_primary_limit_adjustment_view(item: &PrimaryLimitAdjustment) -> PrimaryLimitAdjustmentView {
    PrimaryLimitAdjustmentView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 80),
        ends_at_text: item.ends_at.to_string(),
        host: text::truncate(&item.host, 120),
        updated_at: item.updated_at.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_limit_adjustment_views(items: &[PrimaryLimitAdjustment]) -> Vec<PrimaryLimitAdjustmentView> {
    let mut views: Vec<PrimaryLimitAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_limit_adjustment_view(item));
    }
    views
}

pub fn sum_primary_limit_adjustment_view(view: &PrimaryLimitAdjustmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_updated_at as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_limit_adjustment("code-971", "name-547");
        let view = to_primary_limit_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_limit_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
