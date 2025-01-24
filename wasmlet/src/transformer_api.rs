trait _Plugin {
    fn transform(&self, text: &str) -> Result<String, String>;
}
