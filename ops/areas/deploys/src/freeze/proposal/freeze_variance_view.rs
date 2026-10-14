use super::*;

/// How screens and exports show a freeze variance.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeVarianceView {
    pub code: String,
    pub label_text: String,
    pub amount: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_freeze_variance_view(item: &FreezeVariance) -> FreezeVarianceView {
    FreezeVarianceView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_variance_views(items: &[FreezeVariance]) -> Vec<FreezeVarianceView> {
    let mut views: Vec<FreezeVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_variance_view(item));
    }
    views
}

pub fn sum_freeze_variance_view(view: &FreezeVarianceView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_amount + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_variance("code-27", "label-665");
        let view = to_freeze_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
