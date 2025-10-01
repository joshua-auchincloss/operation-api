use crate::{
    defs::{FromPairSpan, Ident, Spanned},
    *,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaAttribute {
    pub ident: Ident,
    pub style: Style,
    pub raw_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Style {
    Inner,
    Outer,
}

impl FromPairSpan for MetaAttribute {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let mut ident: Option<Ident> = None;
        let mut raw: Option<String> = None;

        let style = match pair.as_rule() {
            Rule::inner_meta => Style::Inner,
            Rule::outer_meta => Style::Outer,
            _ => panic!("unexpected rule"),
        };
        for p in pair
            .into_inner()
            .next()
            .unwrap()
            .into_inner()
        {
            match p.as_rule() {
                Rule::name => {
                    ident = Some(Ident::from(p.as_str()));
                },
                Rule::number | Rule::quoted => raw = Some(p.as_str().to_string()),
                _ => {
                    return Err(crate::Error::defs::<MetaAttribute, _>([
                        crate::Rule::name,
                        crate::Rule::number,
                        crate::Rule::quoted,
                    ]));
                },
            }
        }

        let meta = MetaAttribute {
            style,
            ident: ident
                .ok_or_else(|| crate::Error::def::<MetaAttribute>(crate::Rule::name))
                .map_err(crate::Error::then_with_span(start, end))?,
            raw_value: raw
                .ok_or_else(|| {
                    crate::Error::defs::<MetaAttribute, _>(vec![
                        crate::Rule::number,
                        crate::Rule::quoted,
                    ])
                })
                .map_err(crate::Error::then_with_span(start, end))?,
        };

        Ok(Spanned::new(start, end, meta))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedMeta<T> {
    pub meta: Spanned<MetaAttribute>,
    pub value: T,
}

impl<T: std::str::FromStr> TypedMeta<T> {
    pub fn new_parse(meta: Spanned<MetaAttribute>) -> crate::Result<Self>
    where
        Error: From<<T as std::str::FromStr>::Err>, {
        Ok(Self {
            value: meta.raw_value.parse()?,
            meta,
        })
    }
}

macro_rules! meta {
    ($(
        $kw: ident: $t: ty
    ), + $(,)?) => {

        paste::paste!{
            $(
                pub type [<$kw:camel>] = TypedMeta<$t>;
            )*

            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum Meta {
                $(
                    [<$kw:camel>]([<$kw:camel>])
                )*
            }

            impl Meta {
                pub fn parse(meta: Spanned<MetaAttribute>) -> crate::Result<Self> {
                    #[allow(unused)]
                    if false {}
                    $(
                        else if meta.ident.as_ref() == stringify!([<$kw:snake>]) {
                            return Self::[<from_parse_ $kw:snake>](meta);
                        }
                    )* else {
                        panic!("{}", meta.ident);
                        return Err(crate::Error::def::<MetaAttribute>(crate::Rule::meta_value)
                            .with_span(meta.start, meta.end));
                    }
                    unreachable!("meta parse should have returned or errored above")
                }

                $(

                    pub fn [<from_parse_ $kw:snake>]([<$kw:snake>]: Spanned<MetaAttribute>) -> $crate::Result<Self> {
                        Ok(Self::[<$kw:camel>]([<$kw:camel>]::new_parse([<$kw:snake>])?))
                    }

                    pub fn [<unwrap_ $kw:snake>](self) -> [<$kw:camel>] {
                        match self {
                            Self::[<$kw:camel>](v) => v,
                            #[allow(unreachable_patterns)]
                            _ => panic!("expected {}", stringify!([<$kw:camel>]))
                        }
                    }
                )*
            }
        }
    };
}

meta! {
    Version: usize
}
