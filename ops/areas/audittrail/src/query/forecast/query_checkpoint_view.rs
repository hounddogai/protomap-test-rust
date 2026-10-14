use super::*;

use acme_kit::text;

/// How screens and exports show a query checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryCheckpointView {
    pub code: String,
    pub name: String,
    pub title: String,
    pub summary_text: String,
    pub segment_text: String,
    pub phone_text: String,
    pub duration_minutes: String,
    pub principal: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_query_checkpoint_view(item: &QueryCheckpoint) -> QueryCheckpointView {
    QueryCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        title: item.title.clone(),
        summary_text: item.summary.clone(),
        segment_text: text::truncate(&item.segment, 80),
        phone_text: item.phone.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        principal: text::truncate(&item.principal, 64),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_checkpoint_views(items: &[QueryCheckpoint]) -> Vec<QueryCheckpointView> {
    let mut views: Vec<QueryCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_checkpoint_view(item));
    }
    views
}

pub fn sum_query_checkpoint_view(view: &QueryCheckpointView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_checkpoint("code-855", "name-865");
        let view = to_query_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
