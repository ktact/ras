pub struct StrTab {
    pub data: Vec<u8>,
    pub offsets: Vec<(String, u32)>,
}

impl StrTab {
    pub fn new() -> Self {
        let data = vec![0];
        let offsets = vec![("".to_string(), 0)]; // Store NULL at offset 0
        Self { data, offsets }
    }

    pub fn add_str(&mut self, name: &str) -> u32 {
        if let Some(offset) = self.get_offset_by(name) {
            return offset;
        }

        let offset = self.data.len() as u32;
        self.offsets.push((name.to_string(), offset));
        self.data.extend_from_slice(name.as_bytes());
        self.data.push(0); // Ensure null-termination
        offset
    }

    pub fn get_offset_by(&self, name: &str) -> Option<u32> {
        self.offsets.iter()
            .find(|(stored_name, _)| stored_name == name)
            .map(|(_, offset)| *offset)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
