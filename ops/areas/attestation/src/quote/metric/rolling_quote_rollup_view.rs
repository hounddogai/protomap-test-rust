use super::*;

use acme_kit::text;

/// How screens and exports show a rolling quote rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteRollupView {
    pub code: String,
    pub title_text: String,
    pub floor_value: String,
    pub fingerprint: String,
    pub visible: String,
    pub notes_text: String,
    pub cluster: String,
    pub phone: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_rolling_quote_rollup_view(item: &RollingQuoteRollup) -> RollingQuoteRollupView {
    RollingQuoteRollupView {
        code: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 80),
        floor_value: format!("{:.2}", item.floor_value),
        fingerprint: text::truncate(&item.fingerprint, 80),
        visible: format!("{}", item.visible),
        notes_text: item.notes.clone(),
        cluster: item.cluster.clone(),
        phone: text::truncate(&item.phone, 40),
        summary: text::truncate(&item.summary, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_quote_rollup_views(items: &[RollingQuoteRollup]) -> Vec<RollingQuoteRollupView> {
    let mut views: Vec<RollingQuoteRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_quote_rollup_view(item));
    }
    views
}

pub fn sum_rolling_quote_rollup_view(view: &RollingQuoteRollupView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_quote_rollup("code-604", "title-541");
        let view = to_rolling_quote_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_quote_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
