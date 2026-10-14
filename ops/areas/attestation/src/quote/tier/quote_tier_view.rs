use super::*;

use acme_kit::text;

/// How screens and exports show a quote tier.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTierView {
    pub code: String,
    pub title: String,
    pub grace_days_text: String,
    pub cipher: String,
    pub floor_value_text: String,
    pub locale_text: String,
    pub ends_at: String,
    pub score: String,
    pub archived: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_quote_tier_view(item: &QuoteTier) -> QuoteTierView {
    QuoteTierView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        grace_days_text: item.grace_days.to_string(),
        cipher: text::truncate(&item.cipher, 40),
        floor_value_text: format!("{:.2}", item.floor_value),
        locale_text: text::truncate(&item.locale, 80),
        ends_at: item.ends_at.to_string(),
        score: format!("{:.2}", item.score),
        archived: format!("{}", item.archived),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_tier_views(items: &[QuoteTier]) -> Vec<QuoteTierView> {
    let mut views: Vec<QuoteTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_tier_view(item));
    }
    views
}

pub fn sum_quote_tier_view(view: &QuoteTierView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_floor_value + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_tier("code-72", "title-822");
        let view = to_quote_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
