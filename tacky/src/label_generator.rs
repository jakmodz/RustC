pub struct LabelGenerator {
    label_counter: usize,
}

impl LabelGenerator {
    pub fn new() -> Self {
        LabelGenerator { label_counter: 0 }
    }

    pub fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
}
