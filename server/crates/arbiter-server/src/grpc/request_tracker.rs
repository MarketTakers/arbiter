use tonic::Status;

#[derive(Default)]
pub(super) struct RequestTracker {
    next_request_id: i32,
}

impl RequestTracker {
    pub(super) fn request(&mut self, id: i32) -> Result<i32, Status> {
        if id < self.next_request_id {
            return Err(Status::invalid_argument("Duplicate request id"));
        }

        self.next_request_id = id
            .checked_add(1)
            .ok_or_else(|| Status::invalid_argument("Invalid request id"))?;

        Ok(id)
    }

    // This is used to set the response id for auth responses, which need to match the request id of the auth challenge request.
    // -1 offset is needed because request() increments the next_request_id after returning the current request id.
    pub(super) const fn current_request_id(&self) -> i32 {
        self.next_request_id - 1
    }
}
