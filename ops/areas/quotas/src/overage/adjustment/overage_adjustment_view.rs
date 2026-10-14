use super::*;

use acme_kit::text;

/// How screens and exports show a overage adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustmentView {
    pub code: String,
    pub title: String,
    pub starts_at_text: String,
    pub category: String,
    pub burst_text: String,
    pub audience: String,
    pub key_id: String,
    pub active: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_overage_adjustment_view(item: &OverageAdjustment) -> OverageAdjustmentView {
    OverageAdjustmentView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        starts_at_text: item.starts_at.to_string(),
        category: item.category.clone(),
        burst_text: item.burst.to_string(),
        audience: text::truncate(&item.audience, 40),
        key_id: item.key_id.clone(),
        active: format!("{}", item.active),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_adjustment_views(items: &[OverageAdjustment]) -> Vec<OverageAdjustmentView> {
    let mut views: Vec<OverageAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_adjustment_view(item));
    }
    views
}

pub fn sum_overage_adjustment_view(view: &OverageAdjustmentView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_burst as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_adjustment("code-205", "title-995");
        let view = to_overage_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
