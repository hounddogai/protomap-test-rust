use super::*;

/// How screens and exports show a primary evidence batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceBatchView {
    pub code_text: String,
    pub title_text: String,
    pub amount_text: String,
    pub principal_text: String,
    pub region: String,
    pub host: String,
    pub ratio: String,
    pub pinned: String,
    pub batch_size: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_primary_evidence_batch_view(item: &PrimaryEvidenceBatch) -> PrimaryEvidenceBatchView {
    PrimaryEvidenceBatchView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        amount_text: format!("{:.2}", item.amount),
        principal_text: item.principal.clone(),
        region: item.region.clone(),
        host: item.host.clone(),
        ratio: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        batch_size: item.batch_size.to_string(),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_evidence_batch_views(items: &[PrimaryEvidenceBatch]) -> Vec<PrimaryEvidenceBatchView> {
    let mut views: Vec<PrimaryEvidenceBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_evidence_batch_view(item));
    }
    views
}

pub fn sum_primary_evidence_batch_view(view: &PrimaryEvidenceBatchView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_amount + parsed_ratio + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_evidence_batch("code-12", "title-163");
        let view = to_primary_evidence_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_evidence_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
