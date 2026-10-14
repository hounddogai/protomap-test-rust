use super::*;

/// How screens and exports show a legacy binding slot.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingSlotView {
    pub code: String,
    pub name: String,
    pub fingerprint_text: String,
    pub rate: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_legacy_binding_slot_view(item: &LegacyBindingSlot) -> LegacyBindingSlotView {
    LegacyBindingSlotView {
        code: item.code.clone(),
        name: item.name.clone(),
        fingerprint_text: item.fingerprint.clone(),
        rate: format!("{:.2}", item.rate),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_binding_slot_views(items: &[LegacyBindingSlot]) -> Vec<LegacyBindingSlotView> {
    let mut views: Vec<LegacyBindingSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_binding_slot_view(item));
    }
    views
}

pub fn sum_legacy_binding_slot_view(view: &LegacyBindingSlotView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_binding_slot("code-309", "name-1");
        let view = to_legacy_binding_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_binding_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
