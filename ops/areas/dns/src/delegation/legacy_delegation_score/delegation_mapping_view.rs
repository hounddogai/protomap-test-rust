use super::*;

use acme_kit::text;

/// How screens and exports show a delegation mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMappingView {
    pub code_text: String,
    pub title_text: String,
    pub version: String,
    pub starts_at: String,
    pub retries: String,
    pub source: String,
    pub priority_text: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_delegation_mapping_view(item: &DelegationMapping) -> DelegationMappingView {
    DelegationMappingView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 40),
        version: item.version.to_string(),
        starts_at: item.starts_at.to_string(),
        retries: item.retries.to_string(),
        source: item.source.clone(),
        priority_text: item.priority.to_string(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_mapping_views(items: &[DelegationMapping]) -> Vec<DelegationMappingView> {
    let mut views: Vec<DelegationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_mapping_view(item));
    }
    views
}

pub fn sum_delegation_mapping_view(view: &DelegationMappingView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_version as f64 + (parsed_starts_at as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_mapping("code-163", "title-847");
        let view = to_delegation_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
