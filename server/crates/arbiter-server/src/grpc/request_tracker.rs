use tonic::Status;

#[derive(Default)]
pub struct RequestTracker {
    next_request_id: i32,
}

impl RequestTracker {
    pub fn request(&mut self, id: i32) -> Result<i32, Status> {
        if id < self.next_request_id {
            return Err(Status::invalid_argument("Duplicate request id"));
        }

        self.next_request_id = id
            .checked_add(1)
            .ok_or_else(|| Status::invalid_argument("Invalid request id"))?;

        Ok(id)
    }
}
