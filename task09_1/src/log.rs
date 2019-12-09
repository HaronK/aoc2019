pub struct Log {
    active: bool,
}

impl Log {
    pub fn new(active: bool) -> Self {
        Self { active }
    }

    pub fn print<S: AsRef<str>>(&self, msg: S) {
        if self.active {
            print!("{}", msg.as_ref());
        }
    }
    
    pub fn println<S: AsRef<str>>(&self, msg: S) {
        if self.active {
            println!("{}", msg.as_ref());
        }
    }
}
