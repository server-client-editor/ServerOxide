use crate::user::UserService;

#[derive(Debug)]
pub struct FakeUserService;

impl FakeUserService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl UserService for FakeUserService {

}
