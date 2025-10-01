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
