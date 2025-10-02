use crate::{
    Token,
    ast::comment::{CommentAst, CommentStream},
    defs::Spanned,
    tokens::{Token, tokens},
};

pub struct Import {
    pub kw: Token![import],
    pub path: Spanned<tokens::StringToken>,
}
