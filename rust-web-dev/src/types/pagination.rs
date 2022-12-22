use std::collections::HashMap;

use handle_errors::Error;

/// Pagination struct which is getting extract from query params
// Note `Default` trait usage here to provide default values
#[derive(Default, Debug)]
pub struct Pagination {
    /// Optional max number of items to return
    pub limit: Option<u32>,
    /// The index of the first item which has to be returned
    pub offset: u32,
}

/// Extract query parameters from the `/questions` route
///
/// # Example query
///
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
///
/// `/questions?offset=1&limit=10`
///
/// # Example usage
///
/// ```rust
/// use std::collections::HashMap;
///
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.limit, Some(1));
/// assert_eq!(p.offset, 10);
/// ```
pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    // Needs to check for the existence of just one of the parameters
    // and return an empty value in case nothing is given for "end" and 1 for "start"
    if params.contains_key("offset") && params.contains_key("limit") {
        return Ok(Pagination {
            // Takes the "limit" parameter in the query and tries to convert it to a number
            limit: Some(
                params
                    .get("limit")
                    .unwrap()
                    .parse::<u32>()
                    .map_err(Error::ParseError)?,
            ),
            // Takes the "offset" parameter in the query and tries to convert it to a number
            offset: params
                .get("offset")
                .unwrap()
                .parse::<u32>()
                .map_err(Error::ParseError)?,
        });
    }

    Err(Error::MissingParameters)
}
