pub trait FmtProgress {
    fn get_progress_percentage(&self) -> Option<f64>;
    fn get_estimated_time_remaining(&self) -> Option<f64>;
    fn get_current_speed(&self) -> Option<usize>;

    fn format_progress(&self, message: String) {
        let time = match self.get_estimated_time_remaining() {
            Some(time) => format!("{:.2} ", time),
            None => String::new(),
        };

        let progress = match self.get_progress_percentage() {
            Some(progress) => format!("{:.2}% ", progress),
            None => String::new(),
        };

        let speed = match self.get_current_speed() {
            Some(speed) => {
                match speed  {
                    0..=1023 => format!("{} Bytes/s ", speed),
                    1024..=1048575 => format!("{:.1} KB/s ", speed as f64 / 1024.0),
                    1048576..=1073741823 => format!("{:.1} MB/s ", speed as f64 / (1024.0 * 1024.0)),
                    1073741824..=1099511627775 => format!("{:.1} GB/s ", speed as f64 / (1024.0 * 1024.0 * 1024.0)),
                    _ => format!("{:.1} TB", speed as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)),
                }
            },
            None => String::new(),
        };

        let res = format!(
            "{}{}{}{}",
            time,
            speed,
            progress,
            message
        );
        println!("{}", res);
    }
}