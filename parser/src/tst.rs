use crate::tokens::{self, AstResult, ToTokens, tokenize};

pub fn logging() {
    use std::sync::Once;

    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .pretty()
            .init();
    });
}

pub fn round_trip<T: tokens::Parse + ToTokens>(src: &str) -> AstResult<T> {
    logging();

    let mut tt = tokenize(src)?;

    let t = T::parse(&mut tt)?;
    let tokens = t.tokens();

    let fmt = format!("{tokens}");

    assert_eq!(src, fmt);

    Ok(t)
}

#[allow(unused)]
pub fn basic_smoke<T: tokens::Parse>(src: &str) -> T {
    logging();

    let mut tt = tokenize(src).unwrap();

    T::parse(&mut tt).unwrap()
}
