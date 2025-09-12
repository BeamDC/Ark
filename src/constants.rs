// data units
pub const KILOBYTE: u64 = 1024;
pub const MEGABYTE: u64 = 1048576;
pub const GIGABYTE: u64 = 1073741824;
pub const TERABYTE: u64 = 140737488355328;

#[macro_export] macro_rules! format_bytes {
    ($s: expr) => {
        // not hacky in the slightest
        match $s as u64  {
            0..KILOBYTE => format!("{} bytes", $s),
            KILOBYTE..MEGABYTE => format!("{:.1} KB", $s as f64 / KILOBYTE as f64),
            MEGABYTE..GIGABYTE => format!("{:.1} MB", $s as f64 / MEGABYTE as f64),
            GIGABYTE..TERABYTE => format!("{:.1} GB", $s as f64 / GIGABYTE as f64),
            _ => format!("{:.1} TB", $s as f64 / TERABYTE as f64),
        }
    };
}