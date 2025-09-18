use std::{io::Write, path::PathBuf};

trait MaybeFile: std::io::Write {}

pub struct File {
    f: std::fs::File,
}

pub struct Mem {
    fname: PathBuf,
    fdata: Vec<u8>,
}

pub enum FileOrMem {
    File(File),
    Mem(Mem),
}

impl FileOrMem {
    pub fn new(
        fname: PathBuf,
        mem: bool,
    ) -> std::io::Result<Self> {
        Ok(if !mem {
            let f = std::fs::File::create(fname)?;
            Self::File(File { f })
        } else {
            Self::Mem(Mem {
                fname,
                fdata: vec![],
            })
        })
    }
}

impl Write for FileOrMem {
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(match self {
            Self::File(f) => f.f.flush()?,
            Self::Mem(m) => {},
        })
    }

    fn write(
        &mut self,
        buf: &[u8],
    ) -> std::io::Result<usize> {
        Ok(match self {
            Self::File(f) => f.f.write(buf)?,
            Self::Mem(m) => {
                m.fdata.append(&mut buf.to_vec());
                buf.len()
            },
        })
    }
}
