pub enum ApiError<E> {
    Specific(E),
    Reqwest(reqwest::Error),
}

pub type ApiResult<T, E> = Result<T, ApiError<E>>;

impl<T> From<reqwest::Error> for ApiError<T> {
    fn from(err: reqwest::Error) -> Self {
        ApiError::Reqwest(err)
    }
}
