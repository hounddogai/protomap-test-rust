use super::*;

use acme_kit::text;

/// How screens and exports show a zone window.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneWindowView {
    pub code_text: String,
    pub title: String,
    pub issuer: String,
    pub namespace: String,
    pub checksum: String,
    pub limit_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_zone_window_view(item: &ZoneWindow) -> ZoneWindowView {
    ZoneWindowView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        issuer: item.issuer.clone(),
        namespace: text::truncate(&item.namespace, 120),
        checksum: item.checksum.clone(),
        limit_text: item.limit.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_window_views(items: &[ZoneWindow]) -> Vec<ZoneWindowView> {
    let mut views: Vec<ZoneWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_window_view(item));
    }
    views
}

pub fn sum_zone_window_view(view: &ZoneWindowView) -> Result<f64, String> {
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
        let item = new_zone_window("code-176", "title-445");
        let view = to_zone_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
