use super::*;

/// How screens and exports show a quote batch.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBatchView {
    pub code: String,
    pub name_text: String,
    pub audience: String,
    pub maximum: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_quote_batch_view(item: &QuoteBatch) -> QuoteBatchView {
    QuoteBatchView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        audience: item.audience.clone(),
        maximum: format!("{:.2}", item.maximum),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_batch_views(items: &[QuoteBatch]) -> Vec<QuoteBatchView> {
    let mut views: Vec<QuoteBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_batch_view(item));
    }
    views
}

pub fn sum_quote_batch_view(view: &QuoteBatchView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_batch("code-574", "name-467");
        let view = to_quote_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
