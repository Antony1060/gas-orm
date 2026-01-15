pub trait ModelChangeActor {
    fn forward_sql(&self) -> String;

    fn backward_sql(&self) -> String;
}

// TODO
pub struct SampleModelActor {}

impl ModelChangeActor for SampleModelActor {
    fn forward_sql(&self) -> String {
        "ALTER TABLE foo ADD id BIGINT;".to_string()
    }

    fn backward_sql(&self) -> String {
        "ALTER TABLE foo DROP COLUMN id;".to_string()
    }
}
