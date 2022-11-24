#[macro_export]
macro_rules! bigint {
    ($val : expr) => {
        Into::<BigInt>::into($val)
    };
}
#[macro_export]
macro_rules! mayberelocatable {
    ($val1 : expr, $val2 : expr) => {
        MaybeRelocatable::from(($val1, $val2))
    };
    ($val1 : expr) => {
        MaybeRelocatable::from((bigint!($val1)))
    };
}
#[macro_export]
macro_rules! cairo_runner {
    ($program:expr) => {
        CairoRunner::new(&$program, "all", false).unwrap()
    };
    ($program:expr, $layout:expr) => {
        CairoRunner::new(&$program, $layout, false).unwrap()
    };
    ($program:expr, $layout:expr, $proof_mode:expr) => {
        CairoRunner::new(&$program, $layout, $proof_mode).unwrap()
    };
    ($program:expr, $layout:expr, $proof_mode:expr) => {
        CairoRunner::new(&program, $layout.to_string(), proof_mode).unwrap()
    };
}
#[macro_export]
macro_rules! vm {
    () => {{
        VirtualMachine::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            false,
        )
    }};

    ($use_trace:expr) => {{
        VirtualMachine::new(
            BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            $use_trace,
        )
    }};
}
#[macro_export]
macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None(_) => return,
        }
    }
}
