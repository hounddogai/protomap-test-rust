use super::*;

/// How screens and exports show a silence snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSnapshotView {
    pub code: String,
    pub label_text: String,
    pub currency: String,
    pub cipher: String,
    pub principal: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_silence_snapshot_view(item: &SilenceSnapshot) -> SilenceSnapshotView {
    SilenceSnapshotView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        currency: item.currency.clone(),
        cipher: item.cipher.clone(),
        principal: item.principal.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_snapshot_views(items: &[SilenceSnapshot]) -> Vec<SilenceSnapshotView> {
    let mut views: Vec<SilenceSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_snapshot_view(item));
    }
    views
}

pub fn sum_silence_snapshot_view(view: &SilenceSnapshotView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_snapshot("code-270", "label-92");
        let view = to_silence_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
