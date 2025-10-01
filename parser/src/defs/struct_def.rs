use crate::defs::*;

use pest::iterators::Pairs;

use crate::parser::Rule;

#[derive(Debug, Clone, bon::Builder)]
pub struct Arg<V> {
    pub comment: String,
    pub ident: Ident,
    pub ty: Type<V>,
    pub raw_value: Option<String>,
    pub default_value: Option<Value>,
    pub required: bool,
}

impl<V: Default + Clone + std::fmt::Debug + PartialEq> FromInner for Arg<V> {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut ty: Option<Type<V>> = None;
        let mut default_value = None;
        let mut required = true;
        let mut raw_value = None;
        let mut comment = String::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::arg => return Self::from_inner(pair.into_inner()),
                Rule::name => {
                    let sp = Ident::from_pair_span(pair)?;
                    ident = Some(sp.value);
                },
                Rule::typ => ty = Some(Type::from_inner(Pairs::single(pair))?),
                Rule::eq_value => {
                    if let Some(ref ty) = ty {
                        let inner = pair.into_inner();
                        raw_value = Some(clean_rawvalue(inner.as_str()));
                        default_value = Some(Value::from_inner(inner, ty.clone())?);
                    } else {
                        return Err(crate::Error::defs::<Self, _>([Rule::typ]));
                    }
                },
                Rule::field_sep => {
                    let inner = pair.into_inner().next();
                    match inner.map(|p| p.as_rule()) {
                        Some(Rule::optional_field_sep) => required = false,
                        _ => required = true,
                    }
                },
                Rule::optional_field_sep => {
                    required = false;
                },
                Rule::singleline_comment | Rule::multiline_comment | Rule::comment => {
                    comment += &take_comment(Pairs::single(pair));
                },
                rule => panic!("{rule:#?}"),
            }
        }

        if let Some(ident) = ident
            && let Some(ty) = ty
        {
            Ok(Self {
                comment,
                raw_value,
                ident,
                ty,
                default_value,
                required,
            })
        } else {
            Err(crate::Error::defs::<Self, _>([Rule::ident, Rule::typ]))
        }
    }
}

impl<V: Default + Clone + std::fmt::Debug + PartialEq> FromPairSpan for Arg<V> {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value =
            Arg::from_inner(pair.into_inner()).map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}

#[derive(Debug, Clone, bon::Builder)]
pub struct StructDef<V> {
    pub comment: String,
    pub ident: Ident,
    pub types: Vec<Arg<V>>,
    pub meta: Vec<Meta>,
    pub version: V,
}

impl<V: Clone + std::fmt::Debug + PartialEq + Default> FromInner for StructDef<V> {
    fn from_inner(pairs: Pairs<crate::parser::Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut types = vec![];
        let mut comment = String::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_def => return Self::from_inner(Pairs::single(pair)),
                Rule::ident | Rule::name => {
                    let sp = Ident::from_pair_span(pair)?;
                    ident = Some(sp.value);
                },
                Rule::arg_list => {
                    for arg in pair.into_inner() {
                        let mut comment = String::new();
                        match arg.as_rule() {
                            Rule::multiline_comment => {
                                comment += &take_comment(Pairs::single(arg));
                            },
                            Rule::singleline_comment => {
                                comment += &take_comment(Pairs::single(arg));
                            },
                            _ => {
                                let mut arg = Arg::from_inner(Pairs::single(arg))?;
                                arg.comment += &comment;
                                types.push(arg)
                            },
                        }
                    }
                },
                Rule::comment => {
                    comment += &take_comment(Pairs::single(pair));
                },
                rule => panic!("rule: {rule:#?}"),
            }
        }
        if ident.is_none() {
            return Err(crate::Error::def::<Self>(Rule::ident));
        }
        Ok(Self {
            comment,
            ident: ident.unwrap(),
            types,
            meta: Vec::new(),
            version: V::default(),
        })
    }
}

impl<V> Commentable for StructDef<V> {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

impl<V: Clone + std::fmt::Debug + PartialEq + Default> FromPairSpan for StructDef<V> {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value = StructDef::from_inner(pair.into_inner())
            .map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}

pub type ArgUnsealed = Arg<Option<usize>>;
pub type ArgSealed = Arg<usize>;
pub type StructSealed = StructDef<usize>;
pub type StructUnsealed = StructDef<Option<usize>>;

impl ArgUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> ArgSealed {
        Arg {
            comment: self.comment,
            ident: self.ident,
            ty: self.ty.seal(file_version),
            raw_value: self.raw_value,
            default_value: self.default_value,
            required: self.required,
        }
    }
}
impl StructUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> StructSealed {
        StructDef {
            comment: self.comment,
            ident: self.ident,
            types: self
                .types
                .into_iter()
                .map(|it| it.seal(file_version))
                .collect(),
            meta: self.meta,
            version: self.version.unwrap_or(file_version),
        }
    }
}
