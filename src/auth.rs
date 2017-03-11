
pub trait ObjectAuth {
    fn object_token(&self) -> &String;
    fn is_authorized(&self, token_opt: &Option<String>) -> bool {
        if let Some(ref token) = *token_opt {
            *self.object_token() == *token
        } else {
            false
        }
    }
}
