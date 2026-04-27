pub enum ApiError<E> {
    Specific(E),
    String(String),
    Reqwest(reqwest::Error),
    UrlParse(url::ParseError),
}

pub type ApiResult<T, E> = Result<T, ApiError<E>>;

impl<T> From<reqwest::Error> for ApiError<T> {
    fn from(err: reqwest::Error) -> Self {
        ApiError::Reqwest(err)
    }
}

impl<T> From<url::ParseError> for ApiError<T> {
    fn from(err: url::ParseError) -> Self {
        ApiError::UrlParse(err)
    }
}
