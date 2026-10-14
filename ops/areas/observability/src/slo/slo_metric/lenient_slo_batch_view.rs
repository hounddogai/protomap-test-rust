use super::*;

/// How screens and exports show a lenient slo batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloBatchView {
    pub code: String,
    pub name: String,
    pub ends_at_text: String,
    pub limit: String,
    pub visible_text: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_lenient_slo_batch_view(item: &LenientSloBatch) -> LenientSloBatchView {
    LenientSloBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        ends_at_text: item.ends_at.to_string(),
        limit: item.limit.to_string(),
        visible_text: format!("{}", item.visible),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_slo_batch_views(items: &[LenientSloBatch]) -> Vec<LenientSloBatchView> {
    let mut views: Vec<LenientSloBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_slo_batch_view(item));
    }
    views
}

pub fn sum_lenient_slo_batch_view(view: &LenientSloBatchView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_limit as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_slo_batch("code-62", "name-967");
        let view = to_lenient_slo_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_slo_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
