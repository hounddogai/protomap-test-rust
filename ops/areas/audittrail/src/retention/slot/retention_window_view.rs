use super::*;

use acme_kit::text;

/// How screens and exports show a retention window.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowView {
    pub code_text: String,
    pub name: String,
    pub minimum: String,
    pub batch_size: String,
    pub host: String,
    pub duration_minutes: String,
    pub archived: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_retention_window_view(item: &RetentionWindow) -> RetentionWindowView {
    RetentionWindowView {
        code_text: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 40),
        minimum: format!("{:.2}", item.minimum),
        batch_size: item.batch_size.to_string(),
        host: item.host.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        archived: format!("{}", item.archived),
        cluster_text: text::truncate(&item.cluster, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_window_views(items: &[RetentionWindow]) -> Vec<RetentionWindowView> {
    let mut views: Vec<RetentionWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_window_view(item));
    }
    views
}

pub fn sum_retention_window_view(view: &RetentionWindowView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_minimum + (parsed_batch_size as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_window("code-867", "name-557");
        let view = to_retention_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
