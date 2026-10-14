use super::*;

use acme_kit::text;

/// How screens and exports show a lenient freeze snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshotView {
    pub code: String,
    pub title: String,
    pub batch_size: String,
    pub description: String,
    pub checksum: String,
    pub tenant_id_text: String,
    pub country_text: String,
    pub margin: String,
    pub scope: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_lenient_freeze_snapshot_view(item: &LenientFreezeSnapshot) -> LenientFreezeSnapshotView {
    LenientFreezeSnapshotView {
        code: item.code.clone(),
        title: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        description: text::truncate(&item.description, 80),
        checksum: item.checksum.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 40),
        country_text: text::truncate(&item.country, 80),
        margin: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_freeze_snapshot_views(items: &[LenientFreezeSnapshot]) -> Vec<LenientFreezeSnapshotView> {
    let mut views: Vec<LenientFreezeSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_freeze_snapshot_view(item));
    }
    views
}

pub fn sum_lenient_freeze_snapshot_view(view: &LenientFreezeSnapshotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_freeze_snapshot("code-28", "title-218");
        let view = to_lenient_freeze_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_freeze_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
