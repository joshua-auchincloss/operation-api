use crate::{
    Parse,
    tokens::{self, AstResult, ToTokens, tokenize},
};

pub fn logging() {
    use std::sync::Once;

    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        // miette::set_hook(Box::new(|_| {
        //     Box::new(|e: &miette::Error| {
        //         eprintln!("Error: {}", e);
        //         if let Some(source) = e.source() {
        //             eprintln!("Caused by: {}", source);
        //         }
        //     })
        // }))
        // .expect("Failed to set miette hook");
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .pretty()
            .init();
    });
}

#[macro_export]
macro_rules! assert_matches_debug {
    ($root: literal, $p: ident) => {
        // let expect = include_str!($root);
        let observed = format!("{:#?}", $p);
        std::fs::write(stringify!($root), &observed).unwrap();
        // assert_eq!(expect, observed);
    };
}

pub fn round_trip<T: tokens::Parse + ToTokens>(src: &str) -> AstResult<T> {
    let mut tt = tokenize(src)?;

    let t = T::parse(&mut tt)?;
    let tokens = t.tokens();

    let fmt = format!("{tokens}");

    assert_eq!(src, fmt);

    Ok(t)
}
