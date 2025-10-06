use crate::{
    Parse, Peek, Token,
    defs::Spanned,
    tokens::{Brace, LBraceToken, RBraceToken, Repeated, ToTokens, brace},
};

pub type StructFields = Spanned<Repeated<super::strct::Arg, Token![,]>>;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AnonymousStruct {
    brace: Brace,
    fields: StructFields,
}

impl Parse for AnonymousStruct {
    fn parse(stream: &mut crate::tokens::TokenStream) -> Result<Self, crate::tokens::LexingError> {
        let mut braced;
        Ok(Self {
            brace: brace!(braced in stream),
            fields: braced.parse()?,
        })
    }
}

impl Peek for AnonymousStruct {
    fn peek(stream: &crate::tokens::TokenStream) -> bool {
        let mut fork = stream.fork();

        let mut braced;
        let _ = brace!(braced in fork; false);

        let _: StructFields = crate::bail_unchecked!(braced.parse(); false);

        true
    }
}

impl ToTokens for AnonymousStruct {
    fn write(
        &self,
        tt: &mut crate::tokens::MutTokenStream,
    ) {
        tt.write(&LBraceToken::new());
        tt.write(&self.fields);
        tt.write(&RBraceToken::new());
    }
}

#[cfg(test)]
mod test {
    #[test_case::test_case("{ a: i32 }"; "basic one field")]
    #[test_case::test_case("{ a: i32, b: i64 }"; "basic multi field")]
    #[test_case::test_case("{ a: i32, /* some comment */ b: i64 }"; "fields with comment")]
    pub fn rt_anon(src: &str) {
        crate::tst::round_trip::<super::AnonymousStruct>(src).unwrap();
    }
}
