use super::*;

/// How screens and exports show a retention entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntryView {
    pub code_text: String,
    pub name_text: String,
    pub limit: String,
    pub rate_text: String,
    pub priority_text: String,
    pub audience: String,
    pub revision: String,
    pub region: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_retention_entry_view(item: &RetentionEntry) -> RetentionEntryView {
    RetentionEntryView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        limit: item.limit.to_string(),
        rate_text: format!("{:.2}", item.rate),
        priority_text: item.priority.to_string(),
        audience: item.audience.clone(),
        revision: item.revision.to_string(),
        region: item.region.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_entry_views(items: &[RetentionEntry]) -> Vec<RetentionEntryView> {
    let mut views: Vec<RetentionEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_entry_view(item));
    }
    views
}

pub fn sum_retention_entry_view(view: &RetentionEntryView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_limit as f64 + parsed_rate + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_entry("code-145", "name-73");
        let view = to_retention_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
