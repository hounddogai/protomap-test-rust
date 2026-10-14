use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub ceiling: String,
    pub revision_text: String,
    pub expires_at_text: String,
    pub batch_size_text: String,
    pub name: String,
    pub burst: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_allowlist_cycle_view(item: &AllowlistCycle) -> AllowlistCycleView {
    AllowlistCycleView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        archived: format!("{}", item.archived),
        ceiling: format!("{:.2}", item.ceiling),
        revision_text: item.revision.to_string(),
        expires_at_text: item.expires_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        name: item.name.clone(),
        burst: item.burst.to_string(),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_cycle_views(items: &[AllowlistCycle]) -> Vec<AllowlistCycleView> {
    let mut views: Vec<AllowlistCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_cycle_view(item));
    }
    views
}

pub fn sum_allowlist_cycle_view(view: &AllowlistCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ceiling + (parsed_revision as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_cycle("code-864", "title-298");
        let view = to_allowlist_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
