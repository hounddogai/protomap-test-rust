use super::*;

use acme_kit::text;

/// How screens and exports show a lenient grant cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantCycleView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub archived: String,
    pub quota_units_text: String,
    pub visible: String,
    pub principal: String,
    pub discount: String,
    pub label: String,
    pub cipher: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_lenient_grant_cycle_view(item: &LenientGrantCycle) -> LenientGrantCycleView {
    LenientGrantCycleView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        owner_id: item.owner_id.clone(),
        archived: format!("{}", item.archived),
        quota_units_text: item.quota_units.to_string(),
        visible: format!("{}", item.visible),
        principal: item.principal.clone(),
        discount: format!("{:.2}", item.discount),
        label: item.label.clone(),
        cipher: item.cipher.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_grant_cycle_views(items: &[LenientGrantCycle]) -> Vec<LenientGrantCycleView> {
    let mut views: Vec<LenientGrantCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_grant_cycle_view(item));
    }
    views
}

pub fn sum_lenient_grant_cycle_view(view: &LenientGrantCycleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_discount + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_grant_cycle("code-495", "title-513");
        let view = to_lenient_grant_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_grant_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
