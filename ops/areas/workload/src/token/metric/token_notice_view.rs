use super::*;

/// How screens and exports show a token notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenNoticeView {
    pub code: String,
    pub title_text: String,
    pub created_at: String,
    pub cost: String,
    pub budget: String,
    pub ends_at: String,
    pub unit: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_token_notice_view(item: &TokenNotice) -> TokenNoticeView {
    TokenNoticeView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        created_at: item.created_at.to_string(),
        cost: format!("{:.2}", item.cost),
        budget: format!("{:.2}", item.budget),
        ends_at: item.ends_at.to_string(),
        unit: item.unit.clone(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_notice_views(items: &[TokenNotice]) -> Vec<TokenNoticeView> {
    let mut views: Vec<TokenNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_notice_view(item));
    }
    views
}

pub fn sum_token_notice_view(view: &TokenNoticeView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_cost + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_notice("code-817", "title-963");
        let view = to_token_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
