pub enum Placeholder {
    DollarSign(usize),
    QuestionMark,
}

impl Placeholder {
    pub fn dollar_sign(index: usize) -> Self {
        Placeholder::DollarSign(index)
    }

    pub fn question_mark() -> Self {
        Placeholder::QuestionMark
    }
}

impl Iterator for Placeholder {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        match *self {
            Placeholder::DollarSign(ref mut i) => {
                let r = Some(format!("${}", i));
                *i += 1;
                r
            }
            Placeholder::QuestionMark => Some("?".to_string()),
        }
    }
}
