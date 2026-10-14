use super::*;

use acme_kit::text;

/// How screens and exports show a suspension bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleView {
    pub code_text: String,
    pub label: String,
    pub approved_text: String,
    pub timezone: String,
    pub batch_size: String,
    pub audience_text: String,
    pub amount: String,
    pub starts_at: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_suspension_bundle_view(item: &SuspensionBundle) -> SuspensionBundleView {
    SuspensionBundleView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        approved_text: format!("{}", item.approved),
        timezone: item.timezone.clone(),
        batch_size: item.batch_size.to_string(),
        audience_text: item.audience.clone(),
        amount: format!("{:.2}", item.amount),
        starts_at: item.starts_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_bundle_views(items: &[SuspensionBundle]) -> Vec<SuspensionBundleView> {
    let mut views: Vec<SuspensionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_bundle_view(item));
    }
    views
}

pub fn sum_suspension_bundle_view(view: &SuspensionBundleView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_amount + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_bundle("code-936", "label-162");
        let view = to_suspension_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
