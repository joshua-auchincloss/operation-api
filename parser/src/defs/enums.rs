use crate::defs::*;

use pest::iterators::Pairs;

use Rule;

#[derive(bon::Builder, Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub comment: String,
    pub ident: Ident,
    pub value: Value,
    pub ty: EnumTy,
    pub meta: Vec<MetaAttribute>,
}

impl Commentable for EnumValue {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

impl EnumValue {
    pub fn from_inner(
        pairs: Pairs<Rule>,
        ty: Option<EnumTy>,
        default_value: Value,
    ) -> crate::Result<Self> {
        let mut comment = String::new();
        let mut ident = None;
        let mut value = None;
        let mut ty = ty;
        for pair in pairs {
            match pair.as_rule() {
                Rule::enum_item => {
                    return Self::from_inner(pair.into_inner(), ty, default_value);
                },
                Rule::singleline_comment | Rule::multiline_comment => {
                    comment += &take_comment(Pairs::single(pair));
                },
                Rule::ident | Rule::name => {
                    let sp = Ident::from_pair_span(pair)?;
                    ident = Some(sp.value);
                },
                Rule::eq_value => {
                    match &ty {
                        Some(t) => {
                            let v = Value::from_inner(
                                pair.into_inner(),
                                Type::<Option<usize>>::Builtin(t.builtin()),
                            )?;
                            ty = Some(EnumTy::from_value(&v));
                            value = Some(v);
                        },
                        None => {
                            match Value::from_inner(
                                pair.clone().into_inner(),
                                TypeUnsealed::Builtin(Builtin::I32),
                            ) {
                                Ok(val) => value = Some(val),
                                Err(_) => {
                                    value = Some(Value::from_inner(
                                        pair.into_inner(),
                                        TypeUnsealed::Builtin(Builtin::Str),
                                    )?);
                                    ty = Some(EnumTy::Str);
                                },
                            }
                        },
                    }
                },
                _ => {},
            }
        }
        if ident.is_none() {
            return Err(crate::Error::def::<Self>(Rule::ident));
        }
        Ok(Self {
            comment,
            meta: vec![],
            ident: ident.unwrap(),
            ty: ty.unwrap_or(EnumTy::Int),
            value: value.unwrap_or(default_value),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumTy {
    Int,
    Str,
}

impl EnumTy {
    pub fn builtin(&self) -> Builtin {
        match self {
            EnumTy::Int => Builtin::I32,
            EnumTy::Str => Builtin::Str,
        }
    }

    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::I32(_) => EnumTy::Int,
            Value::Str(_) => EnumTy::Str,
            _ => panic!("EnumTy can only be I32 or Str"),
        }
    }
}

#[derive(bon::Builder, Debug, Clone, PartialEq)]
pub struct EnumDef<V> {
    pub comment: String,
    pub ty: EnumTy,
    pub ident: Ident,
    pub values: Vec<EnumValue>,
    pub meta: Vec<Meta>,
    pub version: V,
}

impl<V> EnumDef<V> {
    pub fn get(
        &self,
        item: Ident,
    ) -> Option<&EnumValue> {
        let obj = item.object().qualified_path();
        self.values
            .iter()
            .find(|it| Ident::path_equals(&obj, &it.ident.qualified_path()))
    }
}

impl<V: Default> FromInner for EnumDef<V> {
    fn from_inner(pairs: Pairs<Rule>) -> crate::Result<Self> {
        let mut ident = None;
        let mut values = Vec::new();
        let mut pending_comment = String::new();
        let mut ty = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::singleline_comment | Rule::multiline_comment => {
                    pending_comment += &take_comment(Pairs::single(pair));
                },
                Rule::ident | Rule::name => {
                    let sp = Ident::from_pair_span(pair)?;
                    ident = Some(sp.value);
                },
                Rule::enum_list => {
                    for (i, v) in pair.into_inner().enumerate() {
                        let default_value = match ty {
                            Some(EnumTy::Int) => Value::I32(0),
                            Some(EnumTy::Str) => Value::Str(String::new()),
                            None => Value::I32(i as i32),
                        };

                        let mut e = EnumValue::from_inner(Pairs::single(v), ty, default_value)?;
                        ty = Some(e.ty.clone());

                        e.comment(pending_comment.clone());
                        pending_comment.clear();
                        values.push(e);
                    }
                },
                _ => {
                    return Err(crate::Error::defs::<Self, _>([
                        Rule::enum_list,
                        Rule::ident,
                        Rule::name,
                        Rule::singleline_comment,
                        Rule::multiline_comment,
                    ]));
                },
            }
        }
        if ident.is_none() {
            return Err(crate::Error::def::<Self>(Rule::ident));
        }
        Ok(Self {
            comment: String::new(),
            ident: ident.unwrap(),
            values,
            ty: ty.unwrap(),
            meta: Vec::new(),
            version: V::default(),
        })
    }
}

impl<V> Commentable for EnumDef<V> {
    fn comment(
        &mut self,
        comment: String,
    ) {
        self.comment += &comment;
    }
}

impl<V: Default> FromPairSpan for EnumDef<V> {
    fn from_pair_span(pair: pest::iterators::Pair<'_, Rule>) -> crate::Result<Spanned<Self>> {
        let span = pair.as_span();
        let start = span.start();
        let end = span.end();
        let value = EnumDef::from_inner(pair.into_inner())
            .map_err(crate::Error::then_with_span(start, end))?;
        Ok(Spanned::new(start, end, value))
    }
}

pub type EnumSealed = EnumDef<usize>;
pub type EnumUnsealed = EnumDef<Option<usize>>;

impl EnumUnsealed {
    pub fn seal(
        self,
        file_version: usize,
    ) -> EnumSealed {
        EnumDef {
            comment: self.comment,
            ty: self.ty,
            ident: self.ident,
            values: self.values,
            meta: self.meta,
            version: self.version.unwrap_or(file_version),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_enum_def() {
        let v1 = EnumValue::builder()
            .comment("".into())
            .ident("a".into())
            .ty(EnumTy::Int)
            .meta(vec![])
            .value(Value::I32(42))
            .build();

        let v2 = EnumValue::builder()
            .comment("".into())
            .ident("b".into())
            .ty(EnumTy::Int)
            .meta(vec![])
            .value(Value::I32(42))
            .build();

        let def = EnumDef::builder()
            .comment("".into())
            .ident("some_enum".into())
            .ty(EnumTy::Int)
            .values(vec![v1, v2])
            .meta(Vec::new())
            .version(1_usize)
            .build();

        let g1 = def.get("some_enum::a".into()).unwrap();
        let g2 = def.get("a".into()).unwrap();
        assert_eq!(g1, g2);
    }
}
