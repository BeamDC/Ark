pub trait FmtProgress {
    fn get_progress_percentage(&self) -> Option<f64>;
    fn get_estimated_time_remaining(&self) -> Option<f64>;

    fn format_progress(&self, message: String) {
        let time = match self.get_estimated_time_remaining() {
            Some(time) => format!("{:.2} ", time),
            None => String::new(),
        };

        let progress = match self.get_progress_percentage() {
            Some(progress) => format!("{:.2}% ", progress),
            None => String::new(),
        };

        let res = format!(
            "{}{}{}",
            time,
            progress,
            message
        );
        println!("{}", res);
    }
}