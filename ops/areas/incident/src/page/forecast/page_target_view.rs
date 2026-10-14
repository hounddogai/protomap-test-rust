use super::*;

use acme_kit::text;

/// How screens and exports show a page target.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTargetView {
    pub code: String,
    pub title_text: String,
    pub burst: String,
    pub amount_text: String,
    pub rate: String,
    pub maximum: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_page_target_view(item: &PageTarget) -> PageTargetView {
    PageTargetView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        burst: item.burst.to_string(),
        amount_text: format!("{:.2}", item.amount),
        rate: format!("{:.2}", item.rate),
        maximum: format!("{:.2}", item.maximum),
        timezone: text::truncate(&item.timezone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_target_views(items: &[PageTarget]) -> Vec<PageTargetView> {
    let mut views: Vec<PageTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_target_view(item));
    }
    views
}

pub fn sum_page_target_view(view: &PageTargetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_burst as f64 + parsed_amount + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_target("code-443", "title-510");
        let view = to_page_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
