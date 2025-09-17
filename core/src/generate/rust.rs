use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::PathBuf,
};

use crate::{
    Operation, Struct,
    generate::{GenOpts, Generate, RustConfig},
};

pub struct RustGenerator;

pub(crate) struct RustGenState {
    opts: GenOpts<RustConfig>,
    files: BTreeMap<PathBuf, Box<dyn Write>>,
}

impl Generate<RustGenState, RustConfig> for RustGenerator {
    fn new_state<'ns>(
        &self,
        opts: &GenOpts<RustConfig>,
    ) -> RustGenState {
        RustGenState {
            files: Default::default(),
            opts: opts.clone(),
        }
    }

    fn gen_operation<'ns>(
        &self,
        state: &mut crate::namespace::WithNsContext<'ns, RustGenState>,
        def: &Operation,
    ) -> super::Result<()> {
        todo!()
    }

    fn gen_struct<'ns>(
        &self,
        state: &mut crate::namespace::WithNsContext<'ns, RustGenState>,
        def: &Struct,
    ) -> super::Result<()> {
        todo!()
    }
}
