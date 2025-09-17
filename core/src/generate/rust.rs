use crate::{
    Operation, Struct,
    generate::{GenOpts, Generate, RustConfig},
};

pub struct RustGenerator;

impl Generate<RustConfig> for RustGenerator {
    fn gen_operation(
        &self,
        def: Operation,
        opts: GenOpts<RustConfig>,
    ) -> super::Result<()> {
        todo!()
    }

    fn gen_struct(
        &self,
        def: Struct,
        opts: GenOpts<RustConfig>,
    ) -> super::Result<()> {
        todo!()
    }
}
