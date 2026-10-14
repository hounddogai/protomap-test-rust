use super::*;

use acme_kit::text;

/// How screens and exports show a suspension record.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecordView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub cluster_text: String,
    pub country: String,
    pub batch_size: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_suspension_record_view(item: &SuspensionRecord) -> SuspensionRecordView {
    SuspensionRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        cluster_text: item.cluster.clone(),
        country: text::truncate(&item.country, 80),
        batch_size: item.batch_size.to_string(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_record_views(items: &[SuspensionRecord]) -> Vec<SuspensionRecordView> {
    let mut views: Vec<SuspensionRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_record_view(item));
    }
    views
}

pub fn sum_suspension_record_view(view: &SuspensionRecordView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_batch_size as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_record("code-615", "name-818");
        let view = to_suspension_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
