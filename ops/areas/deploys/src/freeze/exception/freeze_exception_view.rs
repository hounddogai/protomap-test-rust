use super::*;

use acme_kit::text;

/// How screens and exports show a freeze exception.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeExceptionView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub timezone: String,
    pub unit: String,
    pub checksum: String,
    pub budget_text: String,
    pub burst: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_freeze_exception_view(item: &FreezeException) -> FreezeExceptionView {
    FreezeExceptionView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        timezone: item.timezone.clone(),
        unit: item.unit.clone(),
        checksum: item.checksum.clone(),
        budget_text: format!("{:.2}", item.budget),
        burst: item.burst.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_exception_views(items: &[FreezeException]) -> Vec<FreezeExceptionView> {
    let mut views: Vec<FreezeExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_exception_view(item));
    }
    views
}

pub fn sum_freeze_exception_view(view: &FreezeExceptionView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_rate + parsed_budget + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_exception("code-750", "title-548");
        let view = to_freeze_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
