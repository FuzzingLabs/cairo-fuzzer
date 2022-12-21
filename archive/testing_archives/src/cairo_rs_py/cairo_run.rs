#[cfg(test)]
mod test {
    use crate::cairo_runner::PyCairoRunner;
    use std::fs;

    #[test]
    fn cairo_run_fibonacci() {
        let path = "cairo_programs/fibonacci.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_array_sum() {
        let path = "cairo_programs/array_sum.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner = PyCairoRunner::new(
            program,
            Some("main".to_string()),
            Some(String::from("all")),
            false,
        )
        .unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .expect("Couldn't run program");
    }

    #[test]
    fn cairo_run_hint_print_vars() {
        let path = "cairo_programs/hint_print_vars.json".to_string();
        let program = fs::read_to_string(path).unwrap();
        let mut runner =
            PyCairoRunner::new(program, Some("main".to_string()), None, false).unwrap();
        runner
            .cairo_run_py(false, None, None, None, None, None)
            .expect("Couldn't run program");
    }
}
