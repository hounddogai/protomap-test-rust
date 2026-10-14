use super::*;

use acme_kit::text;

/// How screens and exports show a data export review.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReviewView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub threshold: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_data_export_review_view(item: &DataExportReview) -> DataExportReviewView {
    DataExportReviewView {
        code: item.code.clone(),
        title: item.title.clone(),
        host: item.host.clone(),
        threshold: format!("{:.2}", item.threshold),
        issuer: text::truncate(&item.issuer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_review_views(items: &[DataExportReview]) -> Vec<DataExportReviewView> {
    let mut views: Vec<DataExportReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_review_view(item));
    }
    views
}

pub fn sum_data_export_review_view(view: &DataExportReviewView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_review("code-816", "title-827");
        let view = to_data_export_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
