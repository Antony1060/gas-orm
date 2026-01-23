#[allow(dead_code)]
#[derive(Debug)]
pub struct MigrationScript {
    forwards: &'static str,
    backwards: &'static str,
}

impl MigrationScript {
    pub fn new(forwards: &'static str, backwards: &'static str) -> Self {
        Self {
            forwards,
            backwards,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Migrator<const N: usize> {
    scripts: [MigrationScript; N],
}

impl<const N: usize> Migrator<N> {
    pub fn from(scripts: [MigrationScript; N]) -> Self {
        Migrator { scripts }
    }
}
