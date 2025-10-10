use crate::{
    fmt::{FormatConfig, Printer},
    tokens::{self, AstResult, SpannedToken, ToTokens, tokenize},
};

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
    let mut w = Printer::new(&FormatConfig::default());

    w.write(&t);

    let fmt = w.buf.clone();

    let expected = src.replace("    ", "\t");

    assert_eq!(expected, fmt, "source:\n{src}\ngen:\n{fmt}");

    Ok(t)
}

pub fn basic_smoke<T: tokens::Parse>(src: &str) -> AstResult<T> {
    logging();

    let mut tt = tokenize(src)?;

    let t = T::parse(&mut tt)?;

    Ok(t)
}
