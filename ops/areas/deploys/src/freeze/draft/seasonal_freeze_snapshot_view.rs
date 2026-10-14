use super::*;

/// How screens and exports show a seasonal freeze snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeSnapshotView {
    pub code_text: String,
    pub label: String,
    pub quantity: String,
    pub maximum_text: String,
    pub active: String,
    pub port_text: String,
    pub name: String,
    pub archived: String,
    pub weight: String,
    pub external_ref: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_seasonal_freeze_snapshot_view(item: &SeasonalFreezeSnapshot) -> SeasonalFreezeSnapshotView {
    SeasonalFreezeSnapshotView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        quantity: item.quantity.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        active: format!("{}", item.active),
        port_text: item.port.to_string(),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        weight: format!("{:.2}", item.weight),
        external_ref: item.external_ref.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_freeze_snapshot_views(items: &[SeasonalFreezeSnapshot]) -> Vec<SeasonalFreezeSnapshotView> {
    let mut views: Vec<SeasonalFreezeSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_freeze_snapshot_view(item));
    }
    views
}

pub fn sum_seasonal_freeze_snapshot_view(view: &SeasonalFreezeSnapshotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_maximum + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_freeze_snapshot("code-747", "label-455");
        let view = to_seasonal_freeze_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_freeze_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
