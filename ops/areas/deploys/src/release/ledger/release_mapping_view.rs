use super::*;

use acme_kit::text;

/// How screens and exports show a release mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMappingView {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub currency_text: String,
    pub port_text: String,
    pub ratio: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_release_mapping_view(item: &ReleaseMapping) -> ReleaseMappingView {
    ReleaseMappingView {
        code: item.code.clone(),
        name: item.name.clone(),
        timezone: text::truncate(&item.timezone, 80),
        currency_text: text::truncate(&item.currency, 64),
        port_text: item.port.to_string(),
        ratio: format!("{:.2}", item.ratio),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_mapping_views(items: &[ReleaseMapping]) -> Vec<ReleaseMappingView> {
    let mut views: Vec<ReleaseMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_mapping_view(item));
    }
    views
}

pub fn sum_release_mapping_view(view: &ReleaseMappingView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_port as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_mapping("code-627", "name-328");
        let view = to_release_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
