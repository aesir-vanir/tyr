//! `tyr` macros

macro_rules! not_null {
    ($d:expr, $res:expr) => {
        if $d.null() {
            None
        } else {
            Some($res)
        }
    };
}
