use super::*;

/// How screens and exports show a limit snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotView {
    pub code: String,
    pub label: String,
    pub minimum_text: String,
    pub visible_text: String,
    pub quantity_text: String,
    pub rate: String,
    pub name: String,
    pub version_text: String,
    pub starts_at_text: String,
    pub burst_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_limit_snapshot_view(item: &LimitSnapshot) -> LimitSnapshotView {
    LimitSnapshotView {
        code: item.code.clone(),
        label: item.label.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        visible_text: format!("{}", item.visible),
        quantity_text: item.quantity.to_string(),
        rate: format!("{:.2}", item.rate),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        starts_at_text: item.starts_at.to_string(),
        burst_text: item.burst.to_string(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_snapshot_views(items: &[LimitSnapshot]) -> Vec<LimitSnapshotView> {
    let mut views: Vec<LimitSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_snapshot_view(item));
    }
    views
}

pub fn sum_limit_snapshot_view(view: &LimitSnapshotView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_minimum + (parsed_quantity as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_snapshot("code-300", "label-780");
        let view = to_limit_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
