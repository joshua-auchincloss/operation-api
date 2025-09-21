// we cant export a macro_rule in a proc_macro crate, so we have to include! this file
macro_rules! call_span {
    ($op: expr) => {
        match $op {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        }
    };
    ($op: expr; $err: expr) => {
        match $op {
            Ok(v) => v,
            Err(e) => return ($err)(e).to_compile_error(),
        }
    };
    (@opt $op: expr; $err: expr) => {
        match $op {
            Some(v) => v,
            None => return $err.to_compile_error(),
        }
    };
}
