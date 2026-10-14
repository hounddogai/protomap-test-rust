use super::*;

use acme_kit::text;

/// How screens and exports show a federation batch.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationBatchView {
    pub code: String,
    pub name_text: String,
    pub batch_size: String,
    pub country: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_federation_batch_view(item: &FederationBatch) -> FederationBatchView {
    FederationBatchView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        country: item.country.clone(),
        currency: text::truncate(&item.currency, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_batch_views(items: &[FederationBatch]) -> Vec<FederationBatchView> {
    let mut views: Vec<FederationBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_batch_view(item));
    }
    views
}

pub fn sum_federation_batch_view(view: &FederationBatchView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_batch("code-129", "name-263");
        let view = to_federation_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
