use super::*;

use acme_kit::text;

/// How screens and exports show a token exception.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenExceptionView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub amount_text: String,
    pub visible: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_token_exception_view(item: &TokenException) -> TokenExceptionView {
    TokenExceptionView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 80),
        minimum: format!("{:.2}", item.minimum),
        amount_text: format!("{:.2}", item.amount),
        visible: format!("{}", item.visible),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_exception_views(items: &[TokenException]) -> Vec<TokenExceptionView> {
    let mut views: Vec<TokenExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_exception_view(item));
    }
    views
}

pub fn sum_token_exception_view(view: &TokenExceptionView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_minimum + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_exception("code-663", "name-125");
        let view = to_token_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
